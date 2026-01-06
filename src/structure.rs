#[derive(Clone)]
pub struct Device1D {
    pub x: Vec<f64>,      // メッシュ
    pub nd: Vec<f64>,     // ドナー密度
    pub eps: Vec<f64>,   // 誘電率
    pub phi: Vec<f64>,   // ポテンシャル
}
