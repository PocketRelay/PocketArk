/// Key created from a component and command
pub type ComponentKey = u32;

/// Creates an u32 value from the provided component
/// and command merging them into a single u32
pub const fn component_key(component: u16, command: u16) -> ComponentKey {
    ((component as u32) << 16) + command as u32
}
