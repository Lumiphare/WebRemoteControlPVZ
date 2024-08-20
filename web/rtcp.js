const rtcp = (()=>{
    let pc = new RTCPeerConnection({
        iceServers: [
          {
            urls: "stun:stun.l.google.com:19302",
          },
        ],
      });
      let log = (msg) => {
        document.getElementById("div").innerHTML += msg + "<br>";
      };
    
      let sendChannel = pc.createDataChannel("foo");
      sendChannel.onclose = () => console.log("sendChannel has closed");
      sendChannel.onopen = () => console.log("sendChannel has opened");
      sendChannel.onmessage = (e) =>
        log(`Message from DataChannel '${sendChannel.label}' payload '${e.data}'`);
    
    
      pc.ontrack = function (event) {
        if (event.track.kind === "video") {
          el = document.getElementById("app-screen");
          el.srcObject = event.streams[0]
          el.addEventListener("click", function (e) {
            // 此处是为了移动端特殊处理的点击坐标,如有需要自行修改 
            // var maxX = el.videoWidth;
            // var maxY = el.videoHeight;
            const rect = el.getBoundingClientRect();
            const maxX = rect.height;
            const maxY = rect.width; 
            var x = e.clientY;
            var y = maxY - e.clientX;
            // 打印坐标
            console.log("点击坐标：x=" + x + ", y=" + y);
            console.log(maxX + " " + maxY)
            sendChannel.send(x + " " + y + " " + maxX + " " + maxY);
            // alert(x + " " + y + " " + maxX + " " + maxY);
          });
        } else {
          var el = document.createElement(event.track.kind)
          el.srcObject = event.streams[0]
          el.autoplay = true
          el.controls = true
          document.getElementById('remoteVideos').appendChild(el)
        }
      }
    
      // 当 ICE 连接状态发生变化时触发
      /*
        pc.iceConnectionState 可以是以下几种状态之一：
        new：ICE 代理正在收集候选地址。
        checking：ICE 代理正在检查候选地址的连通性。
        connected：ICE 代理已找到至少一个可用的候选地址对，并建立了连接。
        completed：ICE 代理已完成候选地址的检查，并建立了稳定的连接。
        failed：ICE 代理无法建立连接。
        disconnected：ICE 代理已断开连接。
        closed：ICE 代理已关闭。
      */
      pc.oniceconnectionstatechange = (e) => log(pc.iceConnectionState);
      // 当 ICE 代理收集到新的候选地址时触发
      pc.onicecandidate = (event) => {
        if (event.candidate === null) {
          // pc.localDescription表示本地会话描述
          document.getElementById("localSessionDescription").value = btoa(
            JSON.stringify(pc.localDescription)
          );
          // 请求服务端Session
          window.getSession();
        }
      };
    
      // Offer to receive 1 audio, and 2 video tracks
      pc.addTransceiver("audio", { direction: "recvonly" });
      pc.addTransceiver("video", { direction: "recvonly" });
      // pc.addTransceiver("video", { direction: "recvonly" });
    
      // 创建本地Session描述
      pc.createOffer()
        .then((d) => pc.setLocalDescription(d))
        .catch(log);

      window.startSession = () => {
        let sd = document.getElementById("remoteSessionDescription").value;
        if (sd === "") {
          return alert("Session Description must not be empty");
        }
    
        try {
          pc.setRemoteDescription(new RTCSessionDescription(JSON.parse(atob(sd))));
        } catch (e) {
          alert(e);
        }
      };
})();