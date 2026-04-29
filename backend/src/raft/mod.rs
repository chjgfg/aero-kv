pub mod log_storage;
pub mod network;
pub mod state_machine;
pub mod types;





// 修正后的辅助函数
pub fn to_storage_error<E: std::fmt::Display>(e: E) -> openraft::StorageError<u64> {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, e.to_string());
    
    openraft::StorageError::IO {
        source: openraft::StorageIOError::new(
            openraft::ErrorSubject::Store,
            openraft::ErrorVerb::Read,
            // 关键改动：显式包装成 AnyError
            openraft::AnyError::new(&io_err),
        ),
    }
}