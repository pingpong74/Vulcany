use crate::{
    BufferID, CommandBuffer, Device, ImageID, ImageViewID, SamplerID, Swapchain,
    backend::{device::InnerDevice, swapchain::InnerSwapchain},
    taskgraph::commands::TaskGraphRecordingInterface,
};

use ash::vk;

use std::sync::Arc;

pub enum PassType {
    Graphic,
    Compute,
    Transfer,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ResourceAcess {
    Write,
    Read,
    ReadAndWrite,
}

pub struct PassResource {
    pub buffer: Option<BufferID>,
    pub image: Option<ImageID>,
    pub image_view: Option<ImageViewID>,
    pub sampler: Option<SamplerID>,
    pub acess: ResourceAcess,
}

impl Default for PassResource {
    fn default() -> Self {
        return PassResource {
            buffer: None,
            image: None,
            image_view: None,
            sampler: None,
            acess: ResourceAcess::Write,
        };
    }
}

pub struct Pass {
    pub name: &'static str,
    pub pass_type: PassType,
    pub resources: Vec<PassResource>,
    pub record: fn(&mut CommandBuffer, &Vec<PassResource>),
}

pub struct TaskGraph {
    device: Arc<InnerDevice>,
    swapchain: Arc<InnerSwapchain>,
    recoders: Vec<TaskGraphRecordingInterface>,
    passes: Vec<Pass>,
    edges: Vec<Vec<usize>>,
}

impl TaskGraph {
    pub fn new(device: Device, swapchain: Swapchain) -> TaskGraph {
        let mut tg = TaskGraph {
            device: device.inner.clone(),
            swapchain: swapchain.inner.clone(),
            recoders: Vec::new(),
            passes: Vec::new(),
            edges: Vec::new(),
        };

        tg.create_recording_interfaces();

        return tg;
    }

    pub fn accquire_image(&self) -> (ImageID, ImageViewID) {
        let (index, _) = unsafe {
            self.swapchain
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain.handle,
                    u64::max_value(),
                    vk::Semaphore::null(),
                    vk::Fence::null(),
                )
                .expect("Failed to accquire image")
        };

        return (
            self.swapchain.images[index as usize],
            self.swapchain.image_views[index as usize],
        );
    }

    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }

    pub fn present_accquired_image() {}

    pub fn compile(&self) {
        let edges = TaskGraph::create_adjacency_list(&self.passes);

        for (i, a) in edges.iter().enumerate() {
            print!(
                "Pass name: {} connected to the following: ",
                self.passes[i].name
            );

            for b in a {
                print!(" {}", b);
            }

            println!("");
        }

        let batches = TaskGraph::toplogical_sort(&edges);
        println!("{:?}", batches);
    }

    //Checks if b has a dependency on a
    fn check_dependencies(a: &Pass, b: &Pass) -> bool {
        for res_a in &a.resources {
            for res_b in &b.resources {
                let same_resource = {
                    res_a.buffer.is_some() && res_a.buffer == res_b.buffer
                        || res_a.image.is_some() && res_a.image == res_b.image
                        || res_a.image_view.is_some() && res_a.image_view == res_b.image_view
                        || res_a.sampler.is_some() && res_a.sampler == res_b.sampler
                };

                if same_resource {
                    match (res_a.acess, res_b.acess) {
                        // a writes, b writes
                        (ResourceAcess::Write, ResourceAcess::Write)
                        | (ResourceAcess::ReadAndWrite, _)
                        | (_, ResourceAcess::ReadAndWrite)
                        | (ResourceAcess::Write, ResourceAcess::Read)
                        | (ResourceAcess::Read, ResourceAcess::Write) => return true,

                        _ => {}
                    }
                }
            }
        }
        false
    }

    fn create_adjacency_list(passes: &Vec<Pass>) -> Vec<Vec<usize>> {
        let mut edges = vec![vec![]; passes.len()];

        for (i, pass_i) in passes.iter().enumerate() {
            for j in 0..i {
                let pass_j = &passes[j];

                if TaskGraph::check_dependencies(pass_j, pass_i) {
                    edges[i].push(j);
                }
            }
        }

        TaskGraph::transitive_reduction(&mut edges);

        return edges;
    }

    fn transitive_reduction(adj: &mut Vec<Vec<usize>>) {
        let n = adj.len();

        for u in 0..n {
            let neighbors = adj[u].clone();
            for &v in &neighbors {
                adj[u].retain(|&x| x != v);

                let is_reachable = {
                    let mut visited = vec![false; adj.len()];
                    let mut stack = vec![u];

                    let mut b = false;

                    while let Some(node) = stack.pop() {
                        if node == v {
                            b = true;
                            break;
                        }
                        if visited[node] {
                            continue;
                        }
                        visited[node] = true;
                        for &next in &adj[node] {
                            stack.push(next);
                        }
                    }
                    b
                };

                if !is_reachable {
                    adj[u].push(v);
                }
            }
        }
    }

    //Performs a topological sort using Kahns algorithm
    fn toplogical_sort(adj_list: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
        let mut indegrees = vec![0; adj_list.len()];
        for u in 0..adj_list.len() {
            for &v in &adj_list[u] {
                indegrees[v] += 1;
            }
        }

        let mut q = std::collections::VecDeque::new();
        let mut batches = Vec::new();

        for i in 0..adj_list.len() {
            if indegrees[i] == 0 {
                println!("{}", i);
                q.push_back(i);
            }
        }

        while !q.is_empty() {
            let mut batch = Vec::new();
            let size = q.len();

            for _ in 0..size {
                let u = q.pop_front().unwrap();
                batch.push(u);

                for &v in &adj_list[u] {
                    indegrees[v] -= 1;
                    if indegrees[v] == 0 {
                        q.push_back(v);
                    }
                }
            }

            batches.push(batch);
        }

        batches.reverse();

        return batches;
    }

    fn create_recording_interfaces(&mut self) {
        let queue_families = &self.device.physical_device.queue_families;
        let queue_indices = [
            queue_families.presetation_family.clone().unwrap(),
            queue_families.graphics_family.clone().unwrap(),
            queue_families.transfer_family.clone().unwrap(),
            queue_families.compute_family.clone().unwrap(),
        ];

        for queue_family_index in queue_indices {
            let cmd_pool_create_info =
                vk::CommandPoolCreateInfo::default().queue_family_index(queue_family_index);

            let cmd_pool = unsafe {
                self.device
                    .handle
                    .create_command_pool(&cmd_pool_create_info, None)
                    .expect("Failed to create command pool")
            };

            let cmd_alloc_info = vk::CommandBufferAllocateInfo::default()
                .command_buffer_count(1)
                .command_pool(cmd_pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let cmd_buffer = unsafe {
                self.device
                    .handle
                    .allocate_command_buffers(&cmd_alloc_info)
                    .expect("Failed to allocate command buffer")
            }[0];

            let queue = unsafe { self.device.handle.get_device_queue(queue_family_index, 0) };

            self.recoders.push(TaskGraphRecordingInterface {
                command_pool: cmd_pool,
                command_buffers: vec![cmd_buffer],
                queue_index: queue_family_index,
                queue: queue,
                device: self.device.clone(),
            });
        }
    }
}

impl Drop for TaskGraph {
    fn drop(&mut self) {
        for cmd_pool in &self.recoders {
            unsafe {
                self.device
                    .handle
                    .destroy_command_pool(cmd_pool.command_pool, None);
            }
        }
    }
}
