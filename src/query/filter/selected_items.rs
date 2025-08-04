use crate::data::TransactionData;


#[derive(Debug)]
pub struct SelectedItems {
    pub transaction: bool,
    pub instructions: ItemSelection,
    pub balances: ItemSelection,
    pub token_balances: ItemSelection,
}


impl SelectedItems {
    pub fn new_for_transaction(tx: &TransactionData) -> Self {
        Self {
            transaction: false,
            instructions: ItemSelection::new(tx.instructions.len()),
            balances: ItemSelection::new(tx.balances.len()),
            token_balances: ItemSelection::new(tx.token_balances.len()),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        !self.transaction 
            && self.instructions.is_empty() 
            && self.token_balances.is_empty() 
            && self.balances.is_empty()
    }
}


#[derive(Debug)]
pub struct ItemSelection {
    mask: Vec<bool>,
    len: usize,
    include_all: bool
}


impl ItemSelection {
    pub fn new(len: usize) -> Self {
        Self {
            mask: Vec::new(),
            len,
            include_all: false
        }
    }

    pub fn add_all(&mut self, yes: bool) {
        self.include_all |= yes
    }

    pub fn add(&mut self, i: usize) {
        if self.mask.is_empty() {
            if !self.include_all {
                self.mask.resize(self.len, false);
                self.mask[i] = true
            }
        } else {
            self.mask[i] = true
        }
    }
    
    pub fn includes_all(&self) -> bool {
        self.include_all
    }
    
    pub fn is_empty(&self) -> bool {
        self.mask.is_empty() && (self.len == 0 || !self.include_all)
    }

    pub fn for_each_selected(&self, mut cb: impl FnMut(usize)) {
        if self.include_all {
            (0..self.len).for_each(cb)
        } else {
            self.mask.iter().enumerate().for_each(|(i, include)| {
                if *include {
                    cb(i)
                }
            })
        }
    }
}