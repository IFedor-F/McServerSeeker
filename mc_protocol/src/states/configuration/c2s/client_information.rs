use super::ServerBoundPacket;
use crate::types::{McBool, McVarInt, mc_string};
use bytes::{BufMut, BytesMut};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information

#[derive(Debug)]
pub struct ClientInformationPacket {
    pub locale: String,
    pub view_distance: u8,
    pub chat_mode: i32,
    pub chat_colors: bool,

    // Fields for older protocols (5 - 46)
    pub difficulty: u8,
    pub show_cape: bool,

    // Fields for newer protocols (47+)
    pub displayed_skin_parts: u8,
    pub main_hand: i32,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
    pub particle_status: i32,
}

impl Default for ClientInformationPacket {
    fn default() -> Self {
        Self {
            locale: String::from("en_US"), // set locale to en_US
            view_distance: 8,              // set view distance to 8 chunks
            chat_mode: 0,                  // 0 = enabled
            chat_colors: true,             // enable chat colors
            difficulty: 2,                 // 2 = normal difficulty
            show_cape: true,               // cape visible for old clients
            displayed_skin_parts: 0x7F,    // all parts visible (bitmask)
            main_hand: 1,                  // 1 = Right hand
            enable_text_filtering: false,  // disable text filtering
            allow_server_listings: false,  // don't allow server listings
            particle_status: 0,            // 0 = All particles
        }
    }
}

impl ServerBoundPacket for ClientInformationPacket {
    const MC_NAME: &'static str = "client_information";

    fn encode_payload(self, buf: &mut BytesMut, protocol: i32) {
        match protocol {
            ..=46 => {
                mc_string::write_to_buf(&self.locale, buf);
                buf.put_u8(self.view_distance);
                buf.put_u8(self.chat_mode as u8); // chat_mode was i8 in early versions
                McBool(self.chat_colors).write_to_buf(buf);
                buf.put_u8(self.difficulty);
                McBool(self.show_cape).write_to_buf(buf);
            }
            47..=106 => {
                mc_string::write_to_buf(&self.locale, buf);
                buf.put_u8(self.view_distance);
                buf.put_i8(self.chat_mode as i8);
                McBool(self.chat_colors).write_to_buf(buf);
                buf.put_u8(self.displayed_skin_parts);
            }
            107..=754 => {
                mc_string::write_to_buf(&self.locale, buf);
                buf.put_u8(self.view_distance);
                McVarInt(self.chat_mode).write_to_buf(buf); // chatFlags became VarInt
                McBool(self.chat_colors).write_to_buf(buf);
                buf.put_u8(self.displayed_skin_parts);
                McVarInt(self.main_hand).write_to_buf(buf);
            }
            755..=756 => {
                mc_string::write_to_buf(&self.locale, buf);
                buf.put_u8(self.view_distance);
                McVarInt(self.chat_mode).write_to_buf(buf);
                McBool(self.chat_colors).write_to_buf(buf);
                buf.put_u8(self.displayed_skin_parts);
                McVarInt(self.main_hand).write_to_buf(buf);

                // inverted logic: field was named "disable_text_filtering" in these versions
                McBool(!self.enable_text_filtering).write_to_buf(buf);
            }
            757..=767 => {
                mc_string::write_to_buf(&self.locale, buf);
                buf.put_u8(self.view_distance);
                McVarInt(self.chat_mode).write_to_buf(buf);
                McBool(self.chat_colors).write_to_buf(buf);
                buf.put_u8(self.displayed_skin_parts);
                McVarInt(self.main_hand).write_to_buf(buf);
                McBool(self.enable_text_filtering).write_to_buf(buf);
                McBool(self.allow_server_listings).write_to_buf(buf);
            }
            768.. => {
                mc_string::write_to_buf(&self.locale, buf);
                buf.put_u8(self.view_distance);
                McVarInt(self.chat_mode).write_to_buf(buf);
                McBool(self.chat_colors).write_to_buf(buf);
                buf.put_u8(self.displayed_skin_parts);
                McVarInt(self.main_hand).write_to_buf(buf);
                McBool(self.enable_text_filtering).write_to_buf(buf);
                McBool(self.allow_server_listings).write_to_buf(buf);
                McVarInt(self.particle_status).write_to_buf(buf);
            }
        }
    }
}
