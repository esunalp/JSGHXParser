use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::{BBox, Point3, Vec3};

#[derive(Debug, Clone, Copy)]
struct BvhNode {
    bbox: BBox,
    left: u32,
    right: u32,
    start: u32,
    count: u32,
}

impl BvhNode {
    const fn leaf(bbox: BBox, start: u32, count: u32) -> Self {
        Self {
            bbox,
            left: u32::MAX,
            right: u32::MAX,
            start,
            count,
        }
    }

    const fn inner(bbox: BBox, left: u32, right: u32) -> Self {
        Self {
            bbox,
            left,
            right,
            start: 0,
            count: 0,
        }
    }

    const fn is_leaf(self) -> bool {
        self.count != 0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Bvh {
    nodes: Vec<BvhNode>,
    prim_indices: Vec<u32>,
}

impl Bvh {
    const DEFAULT_LEAF_SIZE: usize = 8;

    #[must_use]
    pub(crate) fn build(bboxes: &[BBox]) -> Option<Self> {
        Self::build_with_leaf_size(bboxes, Self::DEFAULT_LEAF_SIZE)
    }

    #[must_use]
    pub(crate) fn build_with_leaf_size(bboxes: &[BBox], leaf_size: usize) -> Option<Self> {
        if bboxes.is_empty() {
            return None;
        }

        let leaf_size = leaf_size.clamp(1, 256);
        let prim_indices: Vec<u32> = (0..(bboxes.len() as u32)).collect();
        let nodes = Vec::with_capacity(bboxes.len().saturating_mul(2));

        let mut bvh = Self { nodes, prim_indices };
        bvh.build_node(bboxes, 0, bboxes.len(), leaf_size);
        Some(bvh)
    }

    fn build_node(&mut self, bboxes: &[BBox], start: usize, end: usize, leaf_size: usize) -> u32 {
        let node_index = self.nodes.len() as u32;
        let seed_bbox = bboxes[self.prim_indices[start] as usize];
        self.nodes.push(BvhNode::leaf(seed_bbox, 0, 0));

        let bbox = self.range_bbox(bboxes, start, end);
        let count = end - start;

        if count <= leaf_size {
            self.nodes[node_index as usize] = BvhNode::leaf(bbox, start as u32, count as u32);
            return node_index;
        }

        let axis = self.choose_split_axis(bboxes, start, end);
        let mid = start + count / 2;
        self.prim_indices[start..end].select_nth_unstable_by(mid - start, |a, b| {
            let ca = centroid_component(bboxes[*a as usize], axis);
            let cb = centroid_component(bboxes[*b as usize], axis);
            ca.total_cmp(&cb)
        });

        let left = self.build_node(bboxes, start, mid, leaf_size);
        let right = self.build_node(bboxes, mid, end, leaf_size);
        self.nodes[node_index as usize] = BvhNode::inner(bbox, left, right);
        node_index
    }

    fn range_bbox(&self, bboxes: &[BBox], start: usize, end: usize) -> BBox {
        let mut bbox = bboxes[self.prim_indices[start] as usize];
        for &idx in &self.prim_indices[(start + 1)..end] {
            bbox = bbox.union(bboxes[idx as usize]);
        }
        bbox
    }

    fn choose_split_axis(&self, bboxes: &[BBox], start: usize, end: usize) -> u8 {
        let first = bboxes[self.prim_indices[start] as usize].center();
        let mut min = first;
        let mut max = first;

        for &idx in &self.prim_indices[(start + 1)..end] {
            let c = bboxes[idx as usize].center();
            min.x = min.x.min(c.x);
            min.y = min.y.min(c.y);
            min.z = min.z.min(c.z);
            max.x = max.x.max(c.x);
            max.y = max.y.max(c.y);
            max.z = max.z.max(c.z);
        }

        let ex = max.x - min.x;
        let ey = max.y - min.y;
        let ez = max.z - min.z;

        if ex >= ey && ex >= ez {
            0
        } else if ey >= ez {
            1
        } else {
            2
        }
    }

    pub(crate) fn query_bbox<F>(&self, query: BBox, mut visit: F)
    where
        F: FnMut(usize) -> bool,
    {
        if self.nodes.is_empty() {
            return;
        }

        let mut stack = Vec::new();
        stack.push(0u32);

        while let Some(node_idx) = stack.pop() {
            let node = self.nodes[node_idx as usize];
            if !node.bbox.intersects(query) {
                continue;
            }

            if node.is_leaf() {
                let start = node.start as usize;
                let end = start + node.count as usize;
                for &prim in &self.prim_indices[start..end] {
                    if !visit(prim as usize) {
                        return;
                    }
                }
                continue;
            }

            stack.push(node.left);
            stack.push(node.right);
        }
    }

    pub(crate) fn query_ray<F>(
        &self,
        origin: Point3,
        dir: Vec3,
        t_min: f64,
        t_max: f64,
        mut visit: F,
    ) where
        F: FnMut(usize) -> bool,
    {
        if self.nodes.is_empty() {
            return;
        }

        let mut stack = Vec::new();
        stack.push(0u32);

        while let Some(node_idx) = stack.pop() {
            let node = self.nodes[node_idx as usize];
            if !ray_intersects_bbox(origin, dir, node.bbox, t_min, t_max) {
                continue;
            }

            if node.is_leaf() {
                let start = node.start as usize;
                let end = start + node.count as usize;
                for &prim in &self.prim_indices[start..end] {
                    if !visit(prim as usize) {
                        return;
                    }
                }
                continue;
            }

            stack.push(node.left);
            stack.push(node.right);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn nearest<F>(
        &self,
        point: Point3,
        best_dist2: f64,
        mut distance_to_prim: F,
    ) -> Option<(usize, f64)>
    where
        F: FnMut(usize) -> Option<f64>,
    {
        if self.nodes.is_empty() {
            return None;
        }

        let mut best_dist2 = best_dist2;
        let mut best_prim: Option<usize> = None;

        let root_dist2 = bbox_distance_squared_to_point(self.nodes[0].bbox, point);
        let mut heap = BinaryHeap::new();
        heap.push(HeapEntry {
            dist2: root_dist2,
            node: 0u32,
        });

        while let Some(entry) = heap.pop() {
            if entry.dist2 > best_dist2 {
                break;
            }

            let node = self.nodes[entry.node as usize];
            if node.is_leaf() {
                let start = node.start as usize;
                let end = start + node.count as usize;
                for &prim in &self.prim_indices[start..end] {
                    let prim_idx = prim as usize;
                    let Some(d2) = distance_to_prim(prim_idx) else {
                        continue;
                    };
                    if !d2.is_finite() {
                        continue;
                    }
                    if d2 < best_dist2 {
                        best_dist2 = d2;
                        best_prim = Some(prim_idx);
                    }
                }
                continue;
            }

            let left = node.left;
            let right = node.right;

            let left_dist2 = bbox_distance_squared_to_point(self.nodes[left as usize].bbox, point);
            if left_dist2 <= best_dist2 {
                heap.push(HeapEntry {
                    dist2: left_dist2,
                    node: left,
                });
            }

            let right_dist2 = bbox_distance_squared_to_point(self.nodes[right as usize].bbox, point);
            if right_dist2 <= best_dist2 {
                heap.push(HeapEntry {
                    dist2: right_dist2,
                    node: right,
                });
            }
        }

        best_prim.map(|idx| (idx, best_dist2))
    }
}

fn centroid_component(bbox: BBox, axis: u8) -> f64 {
    let c = bbox.center();
    match axis {
        0 => c.x,
        1 => c.y,
        _ => c.z,
    }
}

fn ray_intersects_bbox(origin: Point3, dir: Vec3, bbox: BBox, t_min: f64, t_max: f64) -> bool {
    let mut tmin = t_min;
    let mut tmax = t_max;
    let eps = 1e-15;

    for axis in 0..3u8 {
        let (o, d, min, max) = match axis {
            0 => (origin.x, dir.x, bbox.min.x, bbox.max.x),
            1 => (origin.y, dir.y, bbox.min.y, bbox.max.y),
            _ => (origin.z, dir.z, bbox.min.z, bbox.max.z),
        };

        if !o.is_finite() || !d.is_finite() {
            return false;
        }

        if d.abs() <= eps {
            if o < min || o > max {
                return false;
            }
            continue;
        }

        let inv_d = 1.0 / d;
        let mut t0 = (min - o) * inv_d;
        let mut t1 = (max - o) * inv_d;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        tmin = tmin.max(t0);
        tmax = tmax.min(t1);
        if tmax < tmin {
            return false;
        }
    }

    true
}

#[allow(dead_code)]
fn bbox_distance_squared_to_point(bbox: BBox, point: Point3) -> f64 {
    let dx = if point.x < bbox.min.x {
        bbox.min.x - point.x
    } else if point.x > bbox.max.x {
        point.x - bbox.max.x
    } else {
        0.0
    };
    let dy = if point.y < bbox.min.y {
        bbox.min.y - point.y
    } else if point.y > bbox.max.y {
        point.y - bbox.max.y
    } else {
        0.0
    };
    let dz = if point.z < bbox.min.z {
        bbox.min.z - point.z
    } else if point.z > bbox.max.z {
        point.z - bbox.max.z
    } else {
        0.0
    };
    dx * dx + dy * dy + dz * dz
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct HeapEntry {
    dist2: f64,
    node: u32,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.dist2 == other.dist2 && self.node == other.node
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap acts as a min-heap on dist2.
        other
            .dist2
            .total_cmp(&self.dist2)
            .then_with(|| self.node.cmp(&other.node))
    }
}
