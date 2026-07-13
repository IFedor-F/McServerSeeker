use super::super::types::ReportDetail;
use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McPrefixedArrayField, McReadBuf};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Report_Details
#[derive(Debug)]
pub struct CustomReportDetailsPacket {
    pub details: Vec<ReportDetail>,
}

impl ClientBoundPacket for CustomReportDetailsPacket {
    const MC_NAME: &str = "custom_report_details";

    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let details = McPrefixedArrayField::<ReportDetail>::read_from_buf(&mut data)?;
        Ok(Self { details })
    }
}
