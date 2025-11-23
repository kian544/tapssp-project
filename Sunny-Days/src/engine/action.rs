#[derive(Debug, Clone, Copy)]
pub enum Action {
    Move(i32, i32),

    ToggleInventory,
    InventoryUp,
    InventoryDown,
    UseConsumable,

    Quit,
    None,
}
