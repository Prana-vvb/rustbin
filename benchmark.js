import http from 'k6/http';
import { check } from 'k6';
import { Trend, Counter } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const FILE_SIZE = Number(__ENV.FILE_SIZE || 16 * 1024);

const requestLatency = new Trend('request_latency', true);
const uploadLatency = new Trend('upload_latency', true);
const downloadLatency = new Trend('download_latency', true);

const requestCount = new Counter('request_count');
const errorCount = new Counter('error_count');

const bytesUploaded = new Counter('bytes_uploaded');
const bytesDownloaded = new Counter('bytes_downloaded');

const file = new ArrayBuffer(FILE_SIZE);

export const options = {
    vus: Number(__ENV.VUS || 1),
    duration: __ENV.DURATION || '10s',

    thresholds: {
        http_req_duration: [
            'p(50)<500',
            'p(95)<1000',
            'p(99)<2000',
        ],
        http_req_failed: [
            'rate<0.01',
        ],
    },
};

export default function() {
    // Upload
    const uploadStart = Date.now();

    requestCount.add(1);

    const upload = http.post(
        `${BASE_URL}/data`,
        {
            file: http.file(file, 'benchmark.bin'),
        },
        {
            tags: {
                operation: 'upload',
                name: '/data',
            },
        },
    );

    const uploadTime = Date.now() - uploadStart;

    requestLatency.add(uploadTime);
    uploadLatency.add(uploadTime);

    if (!check(upload, {
        'upload status is 200': (r) => r.status === 200,
    })) {
        errorCount.add(1);
        return;
    }

    bytesUploaded.add(FILE_SIZE);

    const match = upload.body.match(
        /data\/([A-Za-z0-9_-]+)/
    );

    if (!match) {
        errorCount.add(1);
        return;
    }

    // Download
    const downloadStart = Date.now();

    requestCount.add(1);

    const download = http.get(
        `${BASE_URL}/data/${match[1]}`,
        {
            tags: {
                operation: 'download',
                name: '/data/:id',
            },
        },
    );

    const downloadTime = Date.now() - downloadStart;

    requestLatency.add(downloadTime);
    downloadLatency.add(downloadTime);

    if (!check(download, {
        'download status is 200': (r) => r.status === 200,
    })) {
        errorCount.add(1);
        return;
    }

    if (download.body) {
        bytesDownloaded.add(download.body.length);
    }
}
