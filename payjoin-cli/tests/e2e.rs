#[cfg(feature = "_danger-local-https")]
mod e2e {
    use std::process::{ExitStatus, Stdio};

    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    use payjoin_test_utils::{init_bitcoind_sender_receiver, BoxError};
    use tempfile::tempdir;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::process::Command;

    async fn terminate(mut child: tokio::process::Child) -> tokio::io::Result<ExitStatus> {
        let pid = child.id().expect("Failed to get child PID");
        kill(Pid::from_raw(pid as i32), Signal::SIGINT)?;
        // wait for child process to exit completely
        child.wait().await
    }

    const RECEIVE_SATS: &str = "54321";

    #[cfg(feature = "v1")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn send_receive_payjoin_v1() -> Result<(), BoxError> {
        let (bitcoind, _sender, _receiver) = init_bitcoind_sender_receiver(None, None)?;
        let temp_dir = tempdir()?;
        let receiver_db_path = temp_dir.path().join("receiver_db");
        let sender_db_path = temp_dir.path().join("sender_db");
        let port = find_free_port()?;

        let payjoin_sent = tokio::spawn(async move {
            let receiver_rpchost = format!("http://{}/wallet/receiver", bitcoind.params.rpc_socket);
            let sender_rpchost = format!("http://{}/wallet/sender", bitcoind.params.rpc_socket);
            let cookie_file = &bitcoind.params.cookie_file;
            let pj_endpoint = format!("https://localhost:{port}");
            let payjoin_cli = env!("CARGO_BIN_EXE_payjoin-cli");

            let mut cli_receiver = Command::new(payjoin_cli)
                .arg("--bip78")
                .arg("--rpchost")
                .arg(&receiver_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&receiver_db_path)
                .arg("receive")
                .arg(RECEIVE_SATS)
                .arg("--port")
                .arg(port.to_string())
                .arg("--pj-endpoint")
                .arg(&pj_endpoint)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");

            let stdout =
                cli_receiver.stdout.take().expect("Failed to take stdout of child process");
            let reader = BufReader::new(stdout);
            let mut stdout = tokio::io::stdout();
            let mut bip21 = String::new();

            let mut lines = reader.lines();

            while let Some(line) = lines.next_line().await.expect("Failed to read line from stdout")
            {
                // Write to stdout regardless
                stdout
                    .write_all(format!("{line}\n").as_bytes())
                    .await
                    .expect("Failed to write to stdout");

                if line.to_ascii_uppercase().starts_with("BITCOIN") {
                    bip21 = line;
                    break;
                }
            }
            log::debug!("Got bip21 {}", &bip21);

            let mut cli_sender = Command::new(payjoin_cli)
                .arg("--bip78")
                .arg("--rpchost")
                .arg(&sender_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&sender_db_path)
                .arg("send")
                .arg(&bip21)
                .arg("--fee-rate")
                .arg("1")
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");

            let stdout = cli_sender.stdout.take().expect("Failed to take stdout of child process");
            let reader = BufReader::new(stdout);
            let (tx, mut rx) = tokio::sync::mpsc::channel(1);

            let mut lines = reader.lines();
            tokio::spawn(async move {
                let mut stdout = tokio::io::stdout();
                while let Some(line) =
                    lines.next_line().await.expect("Failed to read line from stdout")
                {
                    stdout
                        .write_all(format!("{line}\n").as_bytes())
                        .await
                        .expect("Failed to write to stdout");
                    if line.contains("Payjoin sent") {
                        let _ = tx.send(true).await;
                        break;
                    }
                }
            });

            let timeout = tokio::time::Duration::from_secs(10);
            let payjoin_sent = tokio::time::timeout(timeout, rx.recv())
                .await
                .unwrap_or(Some(false)) // timed out
                .expect("rx channel closed prematurely"); // recv() returned None

            terminate(cli_receiver).await.expect("Failed to kill payjoin-cli");
            terminate(cli_sender).await.expect("Failed to kill payjoin-cli");

            payjoin_sent
        })
        .await?;

        assert!(payjoin_sent, "Payjoin send was not detected");

        fn find_free_port() -> Result<u16, BoxError> {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            Ok(listener.local_addr()?.port())
        }

        Ok(())
    }

    #[cfg(feature = "v2")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn send_receive_payjoin_v2() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use payjoin_test_utils::{init_tracing, TestServices};
        use tempfile::TempDir;
        use tokio::process::{Child, ChildStdout};

        type Result<T> = std::result::Result<T, BoxError>;

        init_tracing();
        let mut services = TestServices::initialize().await?;
        let temp_dir = tempdir()?;

        let result = tokio::select! {
            res = services.take_ohttp_relay_handle() => Err(format!("Ohttp relay is long running: {res:?}").into()),
            res = services.take_directory_handle() => Err(format!("Directory server is long running: {res:?}").into()),
            res = send_receive_cli_async(&services, &temp_dir) => res,
        };

        assert!(result.is_ok(), "send_receive failed: {:#?}", result.unwrap_err());

        async fn send_receive_cli_async(services: &TestServices, temp_dir: &TempDir) -> Result<()> {
            let receiver_db_path = temp_dir.path().join("receiver_db");
            let sender_db_path = temp_dir.path().join("sender_db");
            let (bitcoind, _sender, _receiver) = init_bitcoind_sender_receiver(None, None)?;
            let cert_path = std::env::temp_dir().join("localhost.der");
            tokio::fs::write(&cert_path, services.cert()).await?;
            services.wait_for_services_ready().await?;
            let ohttp_keys = services.fetch_ohttp_keys().await?;
            let ohttp_keys_path = temp_dir.path().join("ohttp_keys");
            tokio::fs::write(&ohttp_keys_path, ohttp_keys.encode()?).await?;

            let receiver_rpchost = format!("http://{}/wallet/receiver", bitcoind.params.rpc_socket);
            let sender_rpchost = format!("http://{}/wallet/sender", bitcoind.params.rpc_socket);
            let cookie_file = &bitcoind.params.cookie_file;

            let payjoin_cli = env!("CARGO_BIN_EXE_payjoin-cli");

            let directory = &services.directory_url().to_string();
            let ohttp_relay = &services.ohttp_relay_url().to_string();

            let cli_receive_initiator = Command::new(payjoin_cli)
                .arg("--rpchost")
                .arg(&receiver_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&receiver_db_path)
                .arg("--ohttp-relays")
                .arg(ohttp_relay)
                .arg("receive")
                .arg(RECEIVE_SATS)
                .arg("--pj-directory")
                .arg(directory)
                .arg("--ohttp-keys")
                .arg(&ohttp_keys_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");
            let bip21 = get_bip21_from_receiver(cli_receive_initiator).await;
            let cli_send_initiator = Command::new(payjoin_cli)
                .arg("--rpchost")
                .arg(&sender_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&sender_db_path)
                .arg("--ohttp-relays")
                .arg(ohttp_relay)
                .arg("send")
                .arg(&bip21)
                .arg("--fee-rate")
                .arg("1")
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");
            send_until_request_timeout(cli_send_initiator).await?;

            let cli_receive_resumer = Command::new(payjoin_cli)
                .arg("--rpchost")
                .arg(&receiver_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&receiver_db_path)
                .arg("--ohttp-relays")
                .arg(ohttp_relay)
                .arg("resume")
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");
            respond_with_payjoin(cli_receive_resumer).await?;

            let cli_send_resumer = Command::new(payjoin_cli)
                .arg("--rpchost")
                .arg(&sender_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&sender_db_path)
                .arg("--ohttp-relays")
                .arg(ohttp_relay)
                .arg("send")
                .arg(&bip21)
                .arg("--fee-rate")
                .arg("1")
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");
            check_payjoin_sent(cli_send_resumer).await?;

            // Check that neither the sender or the receiver have sessions to resume
            let cli_receive_resumer = Command::new(payjoin_cli)
                .arg("--rpchost")
                .arg(&receiver_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&receiver_db_path)
                .arg("--ohttp-relays")
                .arg(ohttp_relay)
                .arg("resume")
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");
            check_resume_has_no_sessions(cli_receive_resumer).await?;
            let cli_send_resumer = Command::new(payjoin_cli)
                .arg("--rpchost")
                .arg(&sender_rpchost)
                .arg("--cookie-file")
                .arg(cookie_file)
                .arg("--db-path")
                .arg(&sender_db_path)
                .arg("--ohttp-relays")
                .arg(ohttp_relay)
                .arg("resume")
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to execute payjoin-cli");
            check_resume_has_no_sessions(cli_send_resumer).await?;
            Ok(())
        }

        async fn get_bip21_from_receiver(mut cli_receiver: Child) -> String {
            let mut stdout =
                cli_receiver.stdout.take().expect("failed to take stdout of child process");
            let bip21 = wait_for_stdout_match(&mut stdout, |line| {
                line.to_ascii_uppercase().starts_with("BITCOIN")
            })
            .await
            .expect("payjoin-cli receiver should output a bitcoin URI");
            log::debug!("Got bip21 {}", &bip21);

            terminate(cli_receiver).await.expect("Failed to kill payjoin-cli");
            bip21
        }

        async fn send_until_request_timeout(mut cli_sender: Child) -> Result<()> {
            let mut stdout =
                cli_sender.stdout.take().expect("failed to take stdout of child process");
            let timeout = tokio::time::Duration::from_secs(35);
            let res = tokio::time::timeout(
                timeout,
                wait_for_stdout_match(&mut stdout, |line| line.contains("No response yet.")),
            )
            .await?;

            terminate(cli_sender).await.expect("Failed to kill payjoin-cli initial sender");
            assert!(res.is_some(), "Fallback send was not detected");
            Ok(())
        }

        async fn respond_with_payjoin(mut cli_receive_resumer: Child) -> Result<()> {
            let mut stdout =
                cli_receive_resumer.stdout.take().expect("Failed to take stdout of child process");
            let timeout = tokio::time::Duration::from_secs(10);
            let res = tokio::time::timeout(
                timeout,
                wait_for_stdout_match(&mut stdout, |line| line.contains("Response successful")),
            )
            .await?;

            terminate(cli_receive_resumer).await.expect("Failed to kill payjoin-cli");
            assert!(res.is_some(), "Did not respond with Payjoin PSBT");
            Ok(())
        }

        async fn check_payjoin_sent(mut cli_send_resumer: Child) -> Result<()> {
            let mut stdout =
                cli_send_resumer.stdout.take().expect("Failed to take stdout of child process");
            let timeout = tokio::time::Duration::from_secs(10);
            let res = tokio::time::timeout(
                timeout,
                wait_for_stdout_match(&mut stdout, |line| line.contains("Payjoin sent")),
            )
            .await?;

            terminate(cli_send_resumer).await.expect("Failed to kill payjoin-cli");
            assert!(res.is_some(), "Payjoin send was not detected");
            Ok(())
        }

        async fn check_resume_has_no_sessions(mut cli_resumer: Child) -> Result<()> {
            let mut stdout =
                cli_resumer.stdout.take().expect("Failed to take stdout of child process");
            let timeout = tokio::time::Duration::from_secs(10);
            let res = tokio::time::timeout(
                timeout,
                wait_for_stdout_match(&mut stdout, |line| line.contains("No sessions to resume.")),
            )
            .await?;

            terminate(cli_resumer).await.expect("Failed to kill payjoin-cli");
            assert!(res.is_some(), "Expected no sessions to resume");
            Ok(())
        }

        /// Read lines from `child_stdout` until `match_pattern` is found and the corresponding
        /// line is returned.
        /// Also writes every read line to tokio::io::stdout();
        async fn wait_for_stdout_match<F>(
            child_stdout: &mut ChildStdout,
            match_pattern: F,
        ) -> Option<String>
        where
            F: Fn(&str) -> bool,
        {
            let reader = BufReader::new(child_stdout);
            let mut lines = reader.lines();
            let mut res = None;

            let mut stdout = tokio::io::stdout();
            while let Some(line) = lines.next_line().await.expect("Failed to read line from stdout")
            {
                // Write all output to tests stdout
                stdout
                    .write_all(format!("{line}\n").as_bytes())
                    .await
                    .expect("Failed to write to stdout");

                if match_pattern(&line) {
                    res = Some(line);
                    break;
                }
            }

            res
        }

        Ok(())
    }
}
