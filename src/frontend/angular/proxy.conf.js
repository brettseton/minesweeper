const { env } = require('process');

const target = env.BACKEND_API_ADDR ? `http://${env.BACKEND_API_ADDR}:${env.BACKEND_PORT}` : 'http://localhost:8080';

const PROXY_CONFIG = [
  {
    context: [
      "/game"
    ],
    target: target,
    secure: false,
    headers: {
      Connection: 'Keep-Alive'
    }
  }
]

module.exports = PROXY_CONFIG;
