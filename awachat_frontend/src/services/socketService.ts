export class SocketService {
  static socket: WebSocket
  static init() {
    this.socket = new WebSocket('ws://127.0.0.1:9000/ws')
    // this.socket.addEventListener('message', (event) => {
    //   console.log(event.data)
    // })
  }
  static sendMessage(msg: string) {
    this.socket.send(msg)
    console.log('发送！')
  }
  static addMessageListener(handler: (this: WebSocket, ev: MessageEvent<string>) => unknown) {
    this.socket.addEventListener('message', handler)
  }
}
