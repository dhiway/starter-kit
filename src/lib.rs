pub mod iroh_core {
    pub mod authors;
    pub mod blobs;
    pub mod docs;
}

pub mod helpers {
    pub mod cli;
    pub mod frontend;
    pub mod state;
    pub mod utils;
}

pub mod node {
    pub mod iroh_wrapper;
}

pub mod api_handlers {
    pub mod blobs_handler;
    pub mod authors_handler;
    pub mod docs_handler;
}

pub mod router {
    pub mod router;
}