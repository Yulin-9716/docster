use rust_xlsxwriter::{Workbook, Format, FormatAlign};
use crate::db::QA;
use deadpool_postgres::Pool;
use std::path::PathBuf;

pub async fn export_to_excel(pool: &Pool, path: PathBuf) -> anyhow::Result<()> {
    let qas = QA::list_all(pool).await?;
    
    let mut workbook = Workbook::new();
    
    let worksheet = workbook.add_worksheet();
    
    let header_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center);

    worksheet.set_column_format(0u16, &header_format)?;
    worksheet.set_column_format(1u16, &header_format)?;
    worksheet.set_column_format(2u16, &header_format)?;
    worksheet.set_column_format(3u16, &header_format)?;
    
    worksheet.write_string(0u32, 0, "ID")?;
    worksheet.write_string(0u32, 1, "问题")?;
    worksheet.write_string(0u32, 2, "回答")?;
    worksheet.write_string(0u32, 3, "创建时间")?;
    
    for (i, qa) in qas.iter().enumerate() {
        let row = (i + 1) as u32;
        worksheet.write_number(row, 0, qa.id as f64)?;
        worksheet.write_string(row, 1, &qa.question)?;
        worksheet.write_string(row, 2, &qa.answer)?;
        worksheet.write_string(row, 3, &qa.created_at.to_string())?;
    }
    
    worksheet.autofit();
    
    workbook.save(&path)?;
    Ok(())
}
