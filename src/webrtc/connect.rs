use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use axum::response;
use axum::routing::Route;
use clap::{AppSettings, Arg, Command};
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_VP8};
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::math_rand_alpha;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use webrtc::Error;

use crate::control::control::post_message_to_control;

pub async fn connect_and_rtp(
    channel: (oneshot::Sender<String>, oneshot::Receiver<String>),
) -> Result<()> {
    // 创建一个媒体ysics引擎对象，用于配置支持的编解码器
    let mut m: MediaEngine = MediaEngine::default();
    m.register_default_codecs()?;

    // 创建一个拦截器注册表。这是用户可配置的RTP/RTCP管道。
    // 这提供了NACKs，RTCP报告等其他功能。如果您使用`webrtc.NewPeerConnection`，则默认启用此功能。
    // 如果您手动管理，则必须为每个PeerConnection创建InterceptorRegistry。
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // 创建API对象，并将媒体引擎传递给它。
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    // 创建本地视频轨道，并将其添加到PeerConnection中。
    let video_track = Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_VP8.to_owned(),
            ..Default::default()
        },
        "video".to_owned(),
        "webrtc-rs".to_owned(),
    ));

    // 将刚刚创建的视频轨道添加到PeerConnection中。
    let rtp_sender = peer_connection
        .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await?;

    // 读取传入的RTCP包
    // 在这些包返回之前，它们会被拦截器处理。例如，NACK需要调用。
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });

    // Open a UDP Listener for RTP Packets on port 5004
    let listener = UdpSocket::bind("127.0.0.1:5004").await?;

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    let done_tx1 = done_tx.clone();

    // 设置ICE连接状态的处理程序，以通知您何时连接/断开与对等方的连接
    peer_connection.on_ice_connection_state_change(Box::new(
        move |connection_state: RTCIceConnectionState| {
            println!("Connection State has changed {connection_state}");
            if connection_state == RTCIceConnectionState::Failed {
                let _ = done_tx1.try_send(());
            }
            Box::pin(async {})
        },
    ));

    let done_tx3 = done_tx.clone();
    // Read RTP packets forever and send them to the WebRTC Client
    let abort_handle = tokio::spawn(async move {
        let mut inbound_rtp_packet = vec![0u8; 1600]; // UDP MTU
        while let Ok((n, _)) = listener.recv_from(&mut inbound_rtp_packet).await {
            if let Err(err) = video_track.write(&inbound_rtp_packet[..n]).await {
                if Error::ErrClosedPipe == err {
                    // The peerConnection has been closed.
                } else {
                    println!("video_track write err: {err}");
                }
                let _ = done_tx3.try_send(());
                return;
            }
        }
    })
    .abort_handle();

    // Register data channel creation handling
    peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
        let d_label = d.label().to_owned();
        let d_id = d.id();
        println!("New DataChannel {d_label} {d_id}");

        // Register channel opening handling
        Box::pin(async move {
            let d2 = Arc::clone(&d);
            let d_label2 = d_label.clone();
            let d_id2 = d_id;
            d.on_close(Box::new(move || {
                println!("Data channel closed");
                Box::pin(async {})
            }));

            d.on_open(Box::new(move || {
                // println!("Data channel '{d_label2}'-'{d_id2}' open. Random messages will now be sent to any connected DataChannels every 5 seconds");
                Box::pin(async move {
                    // let mut result = Result::<usize>::Ok(0);
                    // while result.is_ok() {
                    //     let timeout = tokio::time::sleep(Duration::from_secs(5));
                    //     tokio::pin!(timeout);

                    //     tokio::select! {
                    //         _ = timeout.as_mut() =>{
                    //             let message = math_rand_alpha(15);
                    //             println!("Sending '{message}'");
                    //             result = d2.send_text(message).await.map_err(Into::into);
                    //         }
                    //     };
                    // }
                })
            }));

            // Register text message handling
            d.on_message(Box::new(move |msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                // println!("Message from DataChannel '{d_label}': '{msg_str}'");
                let v_num: Vec<i32> = msg_str
                    .split_whitespace()
                    .map(|s| s.parse::<f32>().expect("Not a number") as i32)
                    .collect();

                post_message_to_control(v_num);
                Box::pin(async {})
            }));
        })
    }));

    let done_tx2 = done_tx.clone();

    // 设置对等连接状态处理程序，以通知您何时连接/断开与对等方的连接。
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");
        if s == RTCPeerConnectionState::Disconnected {
            println!("Peer Connection has gone to closed exiting");
            abort_handle.abort();
        }
        if s == RTCPeerConnectionState::Failed {
            // 等待PeerConnection在30秒内没有网络活动，或另一个故障。它可以使用ICE重新启动重新连接。
            // 可以使用webrtc.PeerConnectionStateDisconnected来检测更快的超时。
            // 请注意，PeerConnection可能从PeerConnectionStateDisconnected恢复。
            println!("Peer Connection has gone to failed exiting: Done forwarding");
            let _ = done_tx2.try_send(());
        }

        Box::pin(async {})
    }));

    // Wait for the offer to be pasted
    // let line = signal::must_read_stdin()?;

    // 从通道中获取客户端的SDP描述
    let line = channel.1.await?;
    let desc_data = signal::decode(line.as_str())?;
    let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;

    // 设置远程SDP描述
    peer_connection.set_remote_description(offer).await?;

    // Create an answer
    let answer = peer_connection.create_answer(None).await?;

    // 创建一个通道，直到ICE收集完成。
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // ICE收集完成后，阻止trickle ICE。我们这样做是因为我们只能在交换一次信号消息。
    // 在生产应用程序中，您应该通过OnICECandidate交换ICE候选者。
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        let b64 = signal::encode(&json_str);
        // println!("{}", b64);
        if let Err(_) = channel.0.send(b64) {
            println!("webrtc sdp send error");
            return Err(anyhow!("send error"));
        }
        // println!("{b64}");
    } else {
        println!("generate local_description failed!");
    }

    println!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };
    peer_connection.close().await?;
    Ok(())
}
