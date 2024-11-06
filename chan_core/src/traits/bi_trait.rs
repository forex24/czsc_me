use crate::common::enums::BiDir;

pub trait BiTrait {
    /// Check if this is a downward bi
    fn is_down(&self) -> bool;
    
    /// Check if this is an upward bi
    fn is_up(&self) -> bool;
    
    /// Get the direction of this bi
    fn dir(&self) -> BiDir;
    
    /// Get the index of this bi
    fn idx(&self) -> usize;
    
    /// Check if this bi is sure/confirmed
    fn is_sure(&self) -> bool;
    
    /// Get the low point value
    fn low(&self) -> f64;
    
    /// Get the high point value
    fn high(&self) -> f64;
    
    /// Set or clear the parent segment reference
    fn set_parent_seg(&mut self, parent: Option</* SegmentHandle type */>) -> bool;
} 