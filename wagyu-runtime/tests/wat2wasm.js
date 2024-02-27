const { spawn } = require('child_process');
const fs = require('fs');

function run(command) {
  const [ f, ...others ] = command.split(" ")

  return new Promise((resolve, reject) => {
    const ps = spawn(f, others);

    let result = "";
    ps.stdout.on('data', (data) => result += data);
    ps.stderr.on('data', (data) => reject(data));
    ps.on('close', (code) => code === 0 ? resolve(result) : reject(code));
  })
}

async function main() {
  const watDirPath = "wagyu-runtime/tests/wat";
  const wasmDirPath = "wagyu-runtime/tests/wasm";

  const watDir = fs.readdirSync(watDirPath);

  for (const watFilename of watDir) {
    await run(`wat2wasm ${watDirPath}/${watFilename} -o ${wasmDirPath}/${watFilename.replace(".wat", ".wasm")}`);
  }
}
main()