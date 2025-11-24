#[derive(Debug, Clone, Copy)]
pub enum Action {
    Move(i32, i32),

    ToggleInventory,
    InventoryUp,
    InventoryDown,
    UseConsumable,

    Confirm, // space/enter to advance title/intro

    Quit,
    None,
}
