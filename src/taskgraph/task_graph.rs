use crate::{BufferID, CommandBuffer, Device, ImageID, Swapchain};

pub enum PassType {
    Graphic,
    Compute,
    Transfer,
}

pub struct ReadResources {
    pub images: Vec<ImageID>,
    pub buffers: Vec<BufferID>,
}

pub struct WriteResources {
    pub images: Vec<ImageID>,
    pub buffers: Vec<BufferID>,
}

pub struct Pass {
    pub name: &'static str,
    pub pass_type: PassType,
    pub read_resources: ReadResources,
    pub write_resources: WriteResources,
    pub record: fn(&mut CommandBuffer, ReadResources, WriteResources),
}

pub struct TaskGraph {
    device: Device,
    swapchain: Swapchain,
    passes: Vec<Pass>,
    edges: Vec<Vec<usize>>,
}

impl TaskGraph {
    pub fn new(device: Device, swapchain: Swapchain) -> TaskGraph {
        return TaskGraph {
            device: device,
            swapchain: swapchain,
            passes: Vec::new(),
            edges: Vec::new(),
        };
    }

    pub fn accquire_image() {}

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
        // Write(A) → Read(B)
        if a.write_resources
            .images
            .iter()
            .any(|id| b.read_resources.images.contains(id))
            || a.write_resources
                .buffers
                .iter()
                .any(|id| b.read_resources.buffers.contains(id))
        {
            return true;
        }

        // Write(A) → Write(B)
        if a.write_resources
            .images
            .iter()
            .any(|id| b.write_resources.images.contains(id))
            || a.write_resources
                .buffers
                .iter()
                .any(|id| b.write_resources.buffers.contains(id))
        {
            return true;
        }

        // Read(A) → Write(B)
        if a.read_resources
            .images
            .iter()
            .any(|id| b.write_resources.images.contains(id))
            || a.read_resources
                .buffers
                .iter()
                .any(|id| b.write_resources.buffers.contains(id))
        {
            return true;
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
}
