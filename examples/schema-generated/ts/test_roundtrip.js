// Basic JS roundtrip test using schema shape
function assert(cond, msg) { if (!cond) throw new Error(msg || 'assertion failed'); }

const node = { id: 'uuid-1', title: 'Alice', position: [1,2,3,4] };
const raw = JSON.stringify(node);
const obj = JSON.parse(raw);
assert(obj.id === node.id, 'id mismatch');
assert(obj.title === node.title, 'title mismatch');
assert(Array.isArray(obj.position) && obj.position.length === 4, 'position mismatch');
console.log('TS/JS roundtrip test passed');
