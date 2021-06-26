use super::*;

pub const ORDER_ID: &[u8] = b"ORD_001";
pub const BILLBOARD_ID: [u8; 4] = 0_u32.to_be_bytes();
pub const CREATIVE_ID: &[u8] = b"video_1.m";
pub const ORDER_DATE: &[u8] = b"ORD_001-20201010";

pub fn sample_data(range: u32) -> (OrderData, Order, SessionData) {
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

	let order_data = OrderData {
		test: b"Some long".to_vec(),
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

	let session_data = SessionData {
		id: b"SD_1".to_vec(),
		order_id: ORDER_ID.to_vec(),
		billboard_id: BILLBOARD_ID.to_vec(),
		creative_id: CREATIVE_ID.to_vec(),
		timestamp: 1614137313,
		date: b"20201010".to_vec(),
		duration: 10
	};

	(order_data, order, session_data)
}
