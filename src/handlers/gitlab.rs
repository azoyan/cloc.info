// #[get("/gitlab.com/{user}/{name}")]
// async fn handle_gitlab(request: HttpRequest, path: web::Path<(String, String)>) -> HttpResponse {
//     let (user, name) = path.into_inner();
//     println!("Request: {request:?}");

//     let url = format!("https://gitlab.com/{user}/{name}");
//     println!("URL: {url}");

//     if let Some(user_agent) = request.headers().get("user-agent") {
//         if let Ok(user_agent_str) = user_agent.to_str() {
//             let parser = Parser::new();
//             if let Some(user_agent) = parser.parse(user_agent_str) {
//                 println!("{user_agent:?}");

//                 if user_agent.os == "UNKNOWN" && user_agent.version == "curl" {
//                     return json_response(&url)
//                         .await
//                         .unwrap_or_else(|e| e.error_response());
//                 }
//             }
//         }
//     }

//     html_respnose(&url)
//         .await
//         .unwrap_or_else(|e| e.error_response())
// }

// #[get("/gitlab.com/{user}/{name}/tree/{branch}")]
// async fn handle_gitlab_branch(
//     request: HttpRequest,
//     path: web::Path<(String, String, String)>,
//     provider: web::Data<GithubProvider>,
// ) -> HttpResponse {
//     let (user, name, branch) = path.into_inner();
//     let provider = provider.get_ref();
//     let url = format!("https://github.com/{user}/{name}/tree/{branch}");
//     let repository = provider.get_with_branch(url, branch);
// }
