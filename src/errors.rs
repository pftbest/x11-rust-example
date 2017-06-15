quick_error! {
    #[derive(Debug)]
    pub enum X11Error {
        OperationFailed(operation: &'static str) {
            from()
        }
    }
}
