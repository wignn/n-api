import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 50 },
    { duration: '1m', target: 100 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<1000'], 
    http_req_failed: ['rate<0.05'],    
  },
};

const BASE_URL = 'http://localhost:4001';

export default function () {
  const response = http.get(`${BASE_URL}/api/books`);
  check(response, { 'status is 200': (r) => r.status === 200 });
}
