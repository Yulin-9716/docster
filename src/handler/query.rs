use std::io::{self, Write};
use crate::{
    Config, 
    chat::{deepseek::ChatClient, Talk, Role, LLMInput, LLMOutput},
    chat::apis::Registry,
    vector_store::VectorStore,
};
use crate::db::QA;

pub async fn handle_query_session(pool: &deadpool_postgres::Pool, config: Config, save: bool) -> anyhow::Result<()> {
    let mut system = config.system_prompt.clone();
    let store = VectorStore::from_config(&config).await?;
    let registry = Registry::new();
    let api_infos = registry.list_apis();
    system.push_str(format!("###以下是提供的API: \n{}###", api_infos).as_str());

    let mut messages = vec![
        Talk {
            role: Role::System,
            content: system,
        },
    ];
    
    let client = ChatClient::from_config(&config);
    let mut input = String::new();

    loop {
        println!("请输入问题 (输入'quit'退出)>>>");
        input.clear();
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") {
            break;
        }

        process_user_input(&client, &registry, &store, &mut messages, input).await?;
        
        if save {
            let output = &messages.last().unwrap().content;
            QA::create(&pool, input, output).await?;
        }
    }

    Ok(())
}

async fn process_user_input(
    client: &ChatClient,
    registry: &Registry,
    store: &VectorStore,
    messages: &mut Vec<Talk>,
    input: &str
) -> anyhow::Result<()> {
    let llm_input = LLMInput {
        content: input.to_string(),
        api_output: false,
    };
    let user_input = serde_json::to_string(&llm_input)?;
    messages.push(Talk::new(Role::User, user_input));
    println!("");

    loop {
        client.get_completion(
            messages, 
            crate::chat::FormatType::JsonObject
        ).await?;

        if let Some(last_msg) = messages.last() {
            let output: LLMOutput = match serde_json::from_str(&last_msg.content) {
                Ok(res) => res,
                Err(err) => {
                    let info = LLMInput::new(err.to_string(), false);
                    let info = serde_json::to_string(&info)?;
                    messages.push(Talk::new(Role::User, info));
                    continue;
                }
            };
            
            if output.call {
                handle_api_call(registry, store, messages, output).await?;
                continue;
            }

            println!("\n{}\n", output.content);
            break;
        } else {
            return Err(anyhow::anyhow!("未知错误"));
        }
    }

    Ok(())
}

async fn handle_api_call(
    registry: &Registry,
    store: &VectorStore,
    messages: &mut Vec<Talk>,
    output: LLMOutput
) -> anyhow::Result<()> {
    println!("{}\n", output.content);
    println!("===智能体正在尝试调用接口...===\n");
    
    let api_name = match output.api {
        Some(api) => api,
        None => {
            println!("===智能体接口调用失败, 重试...===\n");
            let info = LLMInput::new("API名称不能为空".to_string(), true);
            let info = serde_json::to_string(&info)?;
            messages.push(Talk::new(Role::User, info));
            return Ok(());
        },
    };

    println!("===智能体正在调用{}===\n", api_name);
    let params = match output.params {
        Some(params) => params,
        None => {
            println!("===智能体接口调用失败, 重试...===\n");
            let info = LLMInput::new("参数不能为空".to_string(), true);
            let info = serde_json::to_string(&info)?;
            messages.push(Talk::new(Role::User, info));
            return Ok(());
        },
    };

    let output = match registry.handle(store, api_name, params).await {
        Ok(res) => res,
        Err(err) => {
            let info = LLMInput::new(err.to_string(), true);
            let info = serde_json::to_string(&info)?;
            messages.push(Talk::new(Role::User, info));
            return Ok(());
        }
    };

    let output = LLMInput::new(output, true);
    let output = serde_json::to_string(&output)?;
    messages.push(Talk::new(Role::User, output));
    Ok(())
}
