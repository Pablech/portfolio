pub enum Frame {
    Text(String), 
    Skip,         
    Close,        
}

#[derive(PartialEq)]
pub enum Flow {
    Continue,
    Close,
}