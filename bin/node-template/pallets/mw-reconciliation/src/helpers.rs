use super::*;

pub const ORDER_ID: &[u8] = b"ORD_001";
pub const CREATIVE_ID: &[u8] = b"video_1.m";

pub fn sample_data(range: u32) -> (Vec<u8>, OrderData, Order) {
	let size = 10;
	let creative_list = vec![CREATIVE_ID.to_vec(); size];
	let mut target_inventory = Vec::<BillboardData>::new();

	for i in 0..range {
		let id_bytes = i.to_be_bytes();

		let billboard_data = BillboardData {
			id: id_bytes.to_vec(),
			spot_duration: 10,
			spots_per_hour: 100,
			total_spots: 700,
			imp_multiplier_per_day: 1000
		};
		target_inventory.push(billboard_data);
	}

	let order_id = ORDER_ID.to_vec();

	let order_data = OrderData {
		start_date: 1614137312,
		end_date: 1614138312,
		total_spots: 800,
		total_audiences: 50000,
		creative_list,
		target_inventory,
	};

	let order = Order {
		start_date: order_data.start_date,
		end_date: order_data.end_date,
		total_spots: order_data.total_spots,
		total_audiences: order_data.total_audiences,
		creative_list: order_data.creative_list.clone()
	};

	(order_id, order_data, order)
}
