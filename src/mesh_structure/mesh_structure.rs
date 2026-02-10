pub enum IDX {
    Bulk(u32),
    Interface(u32),
}

pub enum FixCharge {
    Bulk(f64),
    Interface(f64),
}

pub struct MeshStructure {
    pub id: Vec<IDX>,
    pub depth: Vec<f64>,
    pub permittivity: Vec<f64>,
    pub dec: Vec<f64>,
    pub nd: Vec<f64>,
    pub end: Vec<f64>,
    pub nc: Vec<f64>,
    pub fixcharge: Vec<FixCharge>,
}

impl MeshStructure {
    pub fn new() -> Self {
        MeshStructure {
            id: vec![],
            depth: vec![],
            permittivity: vec![],
            dec: vec![],
            nd: vec![],
            end: vec![],
            nc: vec![],
            fixcharge: vec![],
        }
    }
}
