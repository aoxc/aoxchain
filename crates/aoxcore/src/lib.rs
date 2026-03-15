// core/src/lib.rs

pub mod block;
pub mod genesis; // 'pub' ekledik, artık node görebilir
pub mod identity; // 'pub' ekledik
pub mod mempool; // 'pub' olduğundan emin ol
pub mod state;
pub mod transaction; // 'pub' olduğundan emin ol
