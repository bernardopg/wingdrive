use uuid::Uuid;
use wingdrive_sdk::prelude::*;

use crate::agent::PhotoEvent;
use crate::models::FaceDetection;
use crate::models::*;

pub fn dbscan_clustering(_faces: &[(Uuid, FaceDetection)], _threshold: f32) -> Vec<FaceCluster> {
	todo!("Implement DBSCAN clustering")
}

pub fn cluster_by_location(_photos: &[Entry], _radius_meters: f32) -> Vec<PlaceCluster> {
	todo!("Implement geographic clustering")
}

pub fn cluster_into_moments(_events: &[PhotoEvent]) -> Vec<MomentGroup> {
	todo!("Implement moment clustering")
}

pub struct FaceCluster {
	pub faces: Vec<(Uuid, FaceDetection)>,
	pub centroid_embedding: Vec<f32>,
}

pub struct PlaceCluster {
	pub photos: Vec<Entry>,
	pub center: GpsCoordinates,
}
