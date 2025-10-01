#[derive(Copy, Clone, PartialEq)]
pub struct BufferID {
    pub(crate) id: u64,
}

#[derive(Copy, Clone, PartialEq)]
pub struct ImageID {
    pub(crate) id: u64,
}

#[derive(Copy, Clone, PartialEq)]
pub struct SamplerID {
    pub(crate) id: u64,
}

#[derive(Copy, Clone, PartialEq)]
pub struct ImageViewID {
    pub(crate) id: u64,
}
