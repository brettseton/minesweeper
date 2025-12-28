const { env } = require('process');

// Use BACKEND_API_ADDR and BACKEND_PORT if set, otherwise default to localhost:8080
const backendAddr = env.BACKEND_API_ADDR || 'localhost';
const backendPort = env.BACKEND_PORT || '8080';
const target = `http://${backendAddr}:${backendPort}`;

console.log(`[Proxy] Target is set to: ${target}`);

const PROXY_CONFIG = [
  {
    context: [
      "/game",
      "/account",
      "/user",
      "/signin-google"
    ],
    target: target,
    secure: false,
    changeOrigin: true,
    logLevel: "debug",
    headers: {
      Connection: 'Keep-Alive'
    }
  }
]

module.exports = PROXY_CONFIG;
