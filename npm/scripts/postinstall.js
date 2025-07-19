const fs = require('fs');
const path = require('path');
const { platform, arch } = process;

// node-fetchを動的にrequireする
let nodeFetch;

try {
  const nodeFetchModule = require('node-fetch');
  nodeFetch = nodeFetchModule.default; // fetchをnodeFetchにリネーム
} catch (e) {
  console.error('node-fetch not found. Please ensure npm install has completed successfully.');
  console.error('Error details:', e.message);
  process.exit(1);
}

const packageName = 'tauria-tsgen'; // package.jsonのnameと合わせる
const owner = 'yamada28go'; // GitHubのリポジトリオーナー名
const repo = 'tauria-tsgen'; // GitHubのリポジトリ名

async function getLatestReleaseVersion() {
  try {
    const response = await nodeFetch(`https://api.github.com/repos/${owner}/${repo}/releases/latest`);
    if (!response.ok) {
      throw new Error(`Failed to fetch latest release: ${response.statusText}`);
    }
    const data = await response.json();
    return data.tag_name.replace(/^v/, ''); // 'v1.0.0' -> '1.0.0' のように 'v' プレフィックスを削除
  } catch (error) {
    console.error(`Error getting latest release version from GitHub API: ${error.message}`);
    // GitHub APIから取得できない場合は、package.jsonのバージョンをフォールバックとして使用
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    if (fs.existsSync(packageJsonPath)) {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
      console.warn(`Falling back to version from package.json: ${packageJson.version}`);
      return packageJson.version;
    }
    throw error; // package.jsonもなければエラー
  }
}

async function downloadBinary() {
  let targetOs = '';
  let targetArch = '';
  let binaryExtension = '';

  switch (platform) {
    case 'linux':
      targetOs = 'linux';
      break;
    case 'darwin':
      targetOs = 'macos';
      break;
    case 'win32':
      targetOs = 'windows';
      binaryExtension = '.exe';
      break;
    default:
      console.error(`Unsupported OS: ${platform}`);
      process.exit(1);
  }

  switch (arch) {
    case 'x64':
      targetArch = 'x64';
      break;
    case 'arm64':
      if (platform === 'darwin') { // macOS (Apple Silicon)
        targetArch = 'aarch64';
      } else {
        console.error(`Unsupported architecture ${arch} for ${platform}`);
        process.exit(1);
      }
      break;
    default:
      console.error(`Unsupported architecture: ${arch}`);
      process.exit(1);
  }

  const version = await getLatestReleaseVersion();
  const assetName = `${packageName}-${targetOs}-${targetArch}${binaryExtension}`;
  const downloadUrl = `https://github.com/${owner}/${repo}/releases/download/v${version}/${assetName}`;
  const binDir = path.join(__dirname, '..', 'bin');
  const binaryPath = path.join(binDir, `${packageName}${binaryExtension}`);

  console.log(`Attempting to download ${packageName} v${version} for ${platform}-${arch}...`);
  console.log(`Download URL: ${downloadUrl}`);

  try {
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    const response = await nodeFetch(downloadUrl);
    if (!response.ok) {
      throw new Error(`Failed to download binary: ${response.statusText} (URL: ${downloadUrl})`);
    }

    const fileStream = fs.createWriteStream(binaryPath);
    await new Promise((resolve, reject) => {
      response.body.pipe(fileStream);
      response.body.on('error', reject);
      fileStream.on('finish', resolve);
    });

    console.log(`Downloaded binary to ${binaryPath}`);

  } catch (error) {
    console.error(`Error during binary download: ${error.message}`);
    console.error('Please ensure your network connection is stable and the binary exists for your platform on GitHub Releases.');
    process.exit(1);
  }
}

async function setExecutablePermissions() {
  const binaryPath = path.join(__dirname, '..', 'bin', packageName + (platform === 'win32' ? '.exe' : ''));
  if (platform !== 'win32') {
    // Windows以外では実行権限を付与
    try {
      fs.chmodSync(binaryPath, '755'); // rwxr-xr-x
      console.log(`Set executable permissions for ${binaryPath}`);
    } catch (error) {
      console.error(`Failed to set executable permissions for ${binaryPath}: ${error.message}`);
      process.exit(1);
    }
  } else {
    console.log('No executable permissions needed for Windows.');
  }
}

async function main() {
  await downloadBinary();
  await setExecutablePermissions();
}

main();