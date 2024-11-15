#[derive(Debug, Clone)]
pub enum Node {
    Box {
        content: BoxContent,

        /// ```text
        ///                     ---  ↑
        ///                    / ,_\ │
        ///                  ,_| |_  │ height
        ///                  |_, ,_| │
        ///                    | |   ↓
        /// reference point ×─ | | ─── baseline
        ///                    | |   ↑
        ///                   /_/    ↓ depth
        ///                  ←─────→
        ///                   width
        /// ```
        size: [f32; 3],
    },
    Glue {
        nat_size: f32,
        shrinkability: f32,
        stretchability: f32,
    },
}

#[derive(Debug, Clone)]
pub enum BoxContent {
    Character(char),
    HRule,
    VRule,
    HBox(Vec<Node>),
    VBox(Vec<Node>),
}
