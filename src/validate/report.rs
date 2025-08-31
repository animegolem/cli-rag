#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub ok: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub doc_count: usize,
    pub id_count: usize,
}
