pub mod vanilla;
pub mod forge;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub filter: bool
}

#[derive(Debug)]
pub struct Version {
    pub id: String,
    pub url: String,
    pub tags: Vec<Tag>
}

#[derive(Debug)]
pub struct Manifest {
    pub tags: Vec<Tag>,
    pub versions: Vec<Version>
}

impl Manifest {
    pub fn get(&self, id: &str) -> Option<&Version> {
        self.versions.iter().find(|v| v.id == id)
    }
}