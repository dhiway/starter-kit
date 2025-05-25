// pub mod authors;
// pub mod blobs;
// pub mod cli;
// pub mod docs;
// pub mod iroh_wrapper;
// pub mod state;
// pub mod utils;

pub mod iroh_core {
    pub mod authors;
    pub mod blobs;
    pub mod docs;
}

pub mod helpers {
    pub mod cli;
    pub mod state;
    pub mod utils;
}

pub mod node {
    pub mod iroh_wrapper;
}

pub mod API_handlers {
    pub mod blobs_handler;
}