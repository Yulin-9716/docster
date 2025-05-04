#[derive(Debug, Clone)]
pub struct Chunk {
    pub content: String,
}

impl Chunk {
    pub fn new(content: String) -> Self {
        Self {
            content,
        }
    }
}

pub fn chunk_document(content: String, chunk_size: usize) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut char_indices = content.char_indices().peekable();
    let mut start = 0;
    
    while char_indices.peek().is_some() {
        let mut end = content.len();
        
        for (i, (pos, _)) in char_indices.by_ref().enumerate() {
            if i == chunk_size {
                end = pos;
                break;
            }
        }
        
        let chunk_content = content[start..end].to_string();
        chunks.push(Chunk::new(chunk_content));
        start = end;
    }
    
    chunks
}

#[cfg(test)]
mod tests {
    use crate::read_config;

    use super::*;

    #[test]
    fn test_chunk() -> anyhow::Result<()> {
        let config = read_config()?;
        let content = "
            哈利·波特从小失去双亲，被寄养在姨妈家里。就像个多余的人，哈利在这个家里得不到丝毫的关爱，只有姨妈一家人的喝斥和欺侮。
            每日睡在碗柜中的哈利多么希望有一天可以离开这个没有温暖的地方，终于在他十一岁生日这天他的愿望实现了。 [21]
        ";
        let result = chunk_document(content.to_string(), config.chunk_size as usize);
        println!("{:?}", result);
        Ok(())
    }
}
