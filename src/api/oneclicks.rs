use crate::DigitalOcean;

pub enum FilterKind {
	Kubernetes,
	Droplet
}

// TODO TODO TODO
impl DigitalOcean {
	pub async fn get_oneclicks(&mut self, kind: FilterKind) {}
}
