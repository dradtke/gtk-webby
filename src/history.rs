const MAX_SIZE: usize = 256;

pub struct History {
    list: Vec<String>,
    cursor: Option<usize>,
}

impl History {
    pub fn new() -> Self {
        History {
            list: Vec::with_capacity(MAX_SIZE as usize),
            cursor: None,
        }
    }

    pub fn push(&mut self, addr: String) {
        self.list.push(addr);
        self.cursor = Some(self.list.len() - 1);
    }

    pub fn can_go_back(&self) -> bool {
        self.cursor.is_some() && self.cursor.unwrap() > 0
    }

    pub fn back(&mut self) -> String {
        if !self.can_go_back() {
            panic!("can't go back!");
        }
        self.cursor = Some(self.cursor.unwrap() - 1);
        self.current_location()
    }

    pub fn can_go_forward(&self) -> bool {
        self.cursor.is_some() && self.cursor.unwrap() < (self.list.len() - 1)
    }

    pub fn forward(&mut self) -> String {
        if !self.can_go_forward() {
            panic!("can't go forward!");
        }
        self.cursor = Some(self.cursor.unwrap() + 1);
        self.current_location()
    }

    fn current_location(&self) -> String {
        self.list.get(self.cursor.unwrap()).unwrap().clone()
    }
}
