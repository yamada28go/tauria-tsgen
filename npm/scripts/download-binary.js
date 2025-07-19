// scripts/download-binary.js
const fs = require('fs');
const path = require('path');
const { platform, arch } = process;
const fetch = require('node-fetch');
const AdmZip = require('adm-zip');

const packageName = 'tauria-tsgen'; // package.jsonのnameと合わせる
const owner = 'yamada28go'; // GitHubのリポジトリオーナー名
const repo = 'tauria-tsgen'; // GitHubのリポジトリ名

async function getLatestReleaseVersion() {
  try {
    const response = await fetch(`https://api.github.com/repos/${owner}/${repo}/releases/latest`);
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
        targetArch = 'arm64';
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
  const assetName = `${packageName}-${targetOs}-${targetArch}.zip`;
  const downloadUrl = `https://github.com/${owner}/${repo}/releases/download/v${version}/${assetName}`;
  const binDir = path.join(__dirname, '..', 'bin');
  const binaryPath = path.join(binDir, `${packageName}${binaryExtension}`);
  const tempZipPath = path.join(binDir, `${assetName}`);

  console.log(`Attempting to download ${packageName} v${version} for ${platform}-${arch}...`);
  console.log(`Download URL: ${downloadUrl}`);

  try {
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    const response = await fetch(downloadUrl);
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
    const extractedFiles = zip.getEntries().map(entry => entry.entryName);
    const extractedBinaryName = extractedFiles.find(name => name.startsWith(packageName) && name.endsWith(binaryExtension));

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

downloadBinary();