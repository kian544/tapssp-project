#[derive(Debug, Clone, Copy)]
pub enum Action {
    Move(i32, i32),

    ToggleInventory,
    ToggleInvTab,   // NEW: T/t cycles inventory tab

    InventoryUp,
    InventoryDown,
    UseConsumable, // also unequip when hovering sword/shield

    ToggleStats,

    Confirm,
    Interact,
    Choice(char),

    // NEW: Battle Option (1=Fight, 2=Inv, 3=Run). bool = 10s penalty active
    BattleOption(u8, bool), 

    Quit, // Ctrl+C / Ctrl+Q
    None,
}