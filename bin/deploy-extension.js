const fs = require('fs').promises;
const {exec} = require('child_process');
const {promisify} = require('util');
const {
    EC2Client,
    DescribeRegionsCommand,
} = require('@aws-sdk/client-ec2');
const {
    LambdaClient,
    PublishLayerVersionCommand,
    AddLayerVersionPermissionCommand,
} = require('@aws-sdk/client-lambda');

const execAsync = promisify(exec);

const ZIP_FILE = './target/lambda/extensions/optimeist-extension.zip';
const EXTENSION_ARM64_ZIP = './target/lambda/extensions/optimeist-extension-arm64.zip';
const EXTENSION_X86_64_ZIP = './target/lambda/extensions/optimeist-extension-x86_64.zip';
const ARNS_FILE = './arns.txt';
const LAYERS_JSON_FILE = './src/layers.json';

const ec2Client = new EC2Client({});

async function getAWSRegions() {
    console.log('Getting a list of all AWS regions...');
    try {
        const command = new DescribeRegionsCommand({});
        const response = await ec2Client.send(command);
        return response.Regions.map(region => region.RegionName).sort((a, b) => a.localeCompare(b, 'en', {numeric: true}));
    } catch (error) {
        console.error('Error getting AWS regions:', error);
        throw error;
    }
}

async function buildExtension(arch, target) {
    console.log(`=== Building extension for --${arch} architecture ===`);
    try {
        const command = `cargo lambda build --release --extension --package optimeist-extension -o zip --target ${target}`;
        await execAsync(command);

        // Move the file to an architecture-specific name
        const targetZip = arch === 'arm64' ? EXTENSION_ARM64_ZIP : EXTENSION_X86_64_ZIP;
        await fs.rename(ZIP_FILE, targetZip);

        return targetZip;
    } catch (error) {
        console.error(`Error building extension for ${arch}:`, error);
        throw error;
    }
}

async function publishLayerToRegion(region, arch, extensionZip) {
    const layerName = `optimeist-extension-${arch}`;
    const lambdaClient = new LambdaClient({region});

    console.log(`Deploying '${layerName}' into '${region}' region`);

    try {
        // Read the zip file
        const zipContent = await fs.readFile(extensionZip);

        // Publish layer version
        const publishCommand = new PublishLayerVersionCommand({
            LayerName: layerName,
            Description: 'Optimeist Extension',
            Content: {
                ZipFile: zipContent,
            },
            CompatibleArchitectures: [arch],
        });

        const publishResponse = await lambdaClient.send(publishCommand);
        const layerVersion = publishResponse.Version;
        const layerArn = publishResponse.LayerVersionArn;

        // Add layer version permission
        const permissionCommand = new AddLayerVersionPermissionCommand({
            LayerName: layerName,
            VersionNumber: layerVersion,
            StatementId: 'public',
            Action: 'lambda:GetLayerVersion',
            Principal: '*',
        });

        await lambdaClient.send(permissionCommand);

        return {region, arch, arn: layerArn};
    } catch (error) {
        console.error(`Error publishing layer to ${region}:`, error);
        throw error;
    }
}

async function publishToAllRegions(arch, extensionZip, regions) {
    console.log(`Using extension zip '${extensionZip}'`);

    const results = [];

    // Process regions sequentially to avoid rate limiting
    for (const region of regions) {
        try {
            const result = await publishLayerToRegion(region, arch, extensionZip);
            results.push(result);
        } catch (error) {
            console.error(`Failed to publish to ${region}:`, error);
            // Continue with other regions
        }
    }

    return results;
}

function convertToLayerMap(results) {
    const layerMap = {};

    results.forEach(({region, arch, arn}) => {
        if (!layerMap[region]) {
            layerMap[region] = {};
        }
        layerMap[region][arch] = arn;
    });

    return layerMap;
}

async function writeResults(results) {
    // Write a markdown file
    let markdownContent = '| Region | Arch | ARN |\n';
    markdownContent += '| -------- | -------- | -------- |\n';

    results.forEach(({region, arch, arn}) => {
        markdownContent += `|${region}|${arch}|${arn}|\n`;
    });

    await fs.writeFile(ARNS_FILE, markdownContent, 'utf8');
    console.log(`Markdown table saved to ${ARNS_FILE}`);

    // Write JSON file
    const layerMap = convertToLayerMap(results);
    await fs.writeFile(LAYERS_JSON_FILE, JSON.stringify(layerMap, null, 2), 'utf8');
    console.log(`JSON layer map saved to ${LAYERS_JSON_FILE}`);
}

async function main() {
    try {
        // Get all AWS regions
        const regions = await getAWSRegions();

        const allResults = [];

        // Build and publish arm64 architecture
        const arm64Zip = await buildExtension('arm64', 'aarch64-unknown-linux-musl');
        const arm64Results = await publishToAllRegions('arm64', arm64Zip, regions);
        allResults.push(...arm64Results);

        // Build and publish x86_64 architecture
        const x86_64Zip = await buildExtension('x86_64', 'x86_64-unknown-linux-musl');
        const x86_64Results = await publishToAllRegions('x86_64', x86_64Zip, regions);
        allResults.push(...x86_64Results);

        // Write results to both markdown and JSON files
        await writeResults(allResults);

        console.log('Deployment completed successfully!');
    } catch (error) {
        console.error('Deployment failed:', error);
        process.exit(1);
    }
}

// Run the main function
main();
