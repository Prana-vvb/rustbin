import http from 'k6/http';
import { check } from 'k6';
import { FormData } from 'https://jslib.k6.io/formdata/0.0.2/index.js';

export const options = {
    vus: 500,
    duration: '60s',
};

const dummyFile = new ArrayBuffer(1024 * 100);

export default function() {
    const url = 'http://127.0.0.1:8080/data';

    let fd = new FormData();
    fd.append('file', http.file(dummyFile, 'benchmark.bin'));

    let postRes = http.post(url, fd.body(), {
        headers: { 'Content-Type': 'multipart/form-data; boundary=' + fd.boundary },
    });

    check(postRes, { 'Upload succeeded (200)': (r) => r.status === 200 });

    if (postRes.status === 200 && postRes.body) {
        let idMatch = postRes.body.match(/data\/([A-Za-z0-9_-]+)/);

        if (idMatch) {
            let fileId = idMatch[1];

            let getRes = http.get(`${url}/${fileId}`);
            check(getRes, { 'Download succeeded (200)': (r) => r.status === 200 });
        }
    }
}
