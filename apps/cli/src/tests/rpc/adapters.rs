use super::*;

fn card(id: &str, installed: Option<&str>, update: bool, consent: bool) -> Value {
	json!({"id":id,"name":id,"summary":"A fixture","description":"Fixture adapter",
		"translations":{},"version":"2.0","author":"Test","license":"MIT","homepage":null,
		"categories":[],"trust":"community","permissions":{"read":["/fixture"],"network":["example.invalid"],"secrets":["token"],"tools":["python"]},
		"platforms":[],"supported_here":true,"runtime_available":true,"installed_version":installed,
		"update_available":update,"update_needs_consent":consent,"revoked_reason":null,"icon_svg":null})
}

fn store(cards: Vec<Value>) -> Value {
	json!({"sequence":42,"isolation":"none","generated_at":null,"adapters":cards,"updates":1})
}

#[tokio::test]
async fn browse_list_and_outdated_use_expected_rpc() {
	for (command, method) in [
		("browse", "action:adapters.store.refresh.input"),
		("list", "query:adapters.store.list"),
		("outdated", "action:adapters.store.refresh.input"),
	] {
		let (result, requests) = run(
			&["wing", "adapters", command],
			OutputFormat::Human,
			vec![
				listing(),
				success(
					method,
					None,
					store(vec![card("alpha", Some("1.0"), true, false)]),
				),
			],
		)
		.await;
		result.unwrap();
		assert_eq!(requests[1], json!({}));
	}
}

#[tokio::test]
async fn install_and_update_forward_consent_and_select_targets() {
	for (command, installed, consent, expected_consent) in [
		("install", None, false, true),
		("update", Some("1.0"), false, false),
		("update", Some("1.0"), true, true),
	] {
		let (result, requests) = run(
			&["wing", "adapters", command, "alpha", "--yes"],
			OutputFormat::Human,
			vec![
				listing(),
				success(
					"action:adapters.store.refresh.input",
					None,
					store(vec![
						card("alpha", installed, true, consent),
						card("other", Some("1.0"), true, false),
					]),
				),
				success(
					"action:adapters.store.install.input",
					None,
					json!({"id":"alpha","version":"2.0","previous_version":installed}),
				),
			],
		)
		.await;
		result.unwrap();
		assert_eq!(
			requests[2],
			json!({"id":"alpha","accept_permissions":expected_consent})
		);
	}
}

#[tokio::test]
async fn missing_install_and_update_arguments_return_errors() {
	for (args, expected) in [
		(
			vec!["wing", "adapters", "install", "missing"],
			"missing is not in the adapter store",
		),
		(
			vec!["wing", "adapters", "update"],
			"Name an adapter or pass --all",
		),
	] {
		let (result, _) = run(
			&args,
			OutputFormat::Human,
			vec![
				listing(),
				success("action:adapters.store.refresh.input", None, store(vec![])),
			],
		)
		.await;
		assert!(result.unwrap_err().to_string().contains(expected));
	}
}

#[tokio::test]
async fn update_all_and_remove_send_expected_payloads() {
	let (result, requests) = run(
		&["wing", "adapters", "update", "--all", "-y"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:adapters.store.refresh.input",
				None,
				store(vec![
					card("alpha", Some("1.0"), true, false),
					card("current", Some("2.0"), false, false),
				]),
			),
			success(
				"action:adapters.store.install.input",
				None,
				json!({"id":"alpha","version":"2.0","previous_version":"1.0"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[2]["id"], "alpha");
	let (result, requests) = run(
		&["wing", "adapters", "remove", "alpha", "-y"],
		OutputFormat::Json,
		vec![
			listing(),
			success(
				"action:adapters.store.remove.input",
				None,
				json!({"id":"alpha"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1], json!({"id":"alpha"}));
}
