const fs = require('fs');
const path = require('path');
const { platform, arch } = process;

// node-fetchとadm-zipを後で設定する
let nodeFetch;
let AdmZip;

const packageName = 'tauria-tsgen'; // package.jsonのnameと合わせる
const owner = 'yamada28go'; // GitHubのリポジトリオーナー名
const repo = 'tauria-tsgen'; // GitHubのリポジトリ名

async function getPackageVersion() {
  const packageJsonPath = path.join(__dirname, '..', 'package.json');
  if (fs.existsSync(packageJsonPath)) {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    return packageJson.version;
  }
  throw new Error('package.json not found.');
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

  const version = await getPackageVersion();
  const assetName = `${packageName}-${targetOs}-${targetArch}.zip`; // ZIPファイル名に変更
  const downloadUrl = `https://github.com/${owner}/${repo}/releases/download/v${version}/${assetName}`;
  const binDir = path.join(__dirname, '..', 'bin');
  const binaryPath = path.join(binDir, `${packageName}${binaryExtension}`);
  const tempZipPath = path.join(binDir, `${assetName}`); // 一時ZIPファイルのパス

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

    const fileStream = fs.createWriteStream(tempZipPath);
    await new Promise((resolve, reject) => {
      response.body.pipe(fileStream);
      response.body.on('error', reject);
      fileStream.on('finish', resolve);
    });

    console.log(`Downloaded ${assetName} to ${tempZipPath}`);

    // ZIPファイルを解凍
    const zip = new AdmZip(tempZipPath);
    zip.extractAllTo(binDir, true); // trueは上書きを許可

    // 解凍後のファイル名を特定し、適切な名前にリネーム
    // ZIPファイルには通常、単一のバイナリが含まれると仮定
    const expectedBinaryPrefix = `${packageName}-${targetOs}-${targetArch}`;
    const extractedFiles = zip.getEntries().map(entry => entry.entryName);
    const extractedBinaryName = extractedFiles.find(name => name.startsWith(expectedBinaryPrefix) && name.endsWith(binaryExtension));

    if (extractedBinaryName) {
      fs.renameSync(path.join(binDir, extractedBinaryName), binaryPath);
      console.log(`Extracted and moved binary to ${binaryPath}`);
    } else {
      throw new Error(`Could not find binary in downloaded zip: ${extractedFiles.join(', ')}`);
    }

    fs.unlinkSync(tempZipPath); // 一時ZIPファイルを削除
    console.log('Binary download and extraction complete.');

  } catch (error) {
    console.error(`Error during binary download or extraction: ${error.message}`);
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

function createBinSymlink() {
  // __dirname = .../node_modules/tauria-tsgen/scripts
  const pkgRoot   = path.resolve(__dirname, '..');            // .../node_modules/tauria-tsgen
  const nmDir     = path.resolve(pkgRoot, '..');              // .../node_modules
  const linkDir   = path.join(nmDir, '.bin');                 // .../node_modules/.bin
  const targetBin = path.join(pkgRoot, 'bin', packageName + (platform === 'win32' ? '.exe' : ''));
  const linkPath  = path.join(linkDir, packageName);

  if (!fs.existsSync(linkDir)) fs.mkdirSync(linkDir, { recursive: true });
  if (fs.existsSync(linkPath)) fs.unlinkSync(linkPath);

  fs.symlinkSync(targetBin, linkPath, 'file');
  console.log(`Linked: ${linkPath} → ${targetBin}`);
}

async function main() {
  try {
    const nodeFetchModule = await import('node-fetch');
    nodeFetch = nodeFetchModule.default;
    AdmZip = require('adm-zip');
  } catch (e) {
    console.error('node-fetch or adm-zip not found. Please ensure npm install has completed successfully.');
    console.error('Error details:', e.message);
    process.exit(1);
  }

  try {
    await downloadBinary();
    await setExecutablePermissions();
    createBinSymlink();
    console.log('✅ postinstall 完了');
  } catch (err) {
    console.error('postinstall エラー:', err);
    process.exit(1);
  }
}

main();