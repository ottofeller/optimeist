import * as cdk from 'aws-cdk-lib'

export const layersMap: Record<typeof cdk.Aws.REGION, Record<'arm64' | 'x86_64', string>> = {
  'ap-northeast-1': {
    arm64: 'arn:aws:lambda:ap-northeast-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:ap-northeast-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'ap-northeast-2': {
    arm64: 'arn:aws:lambda:ap-northeast-2:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:ap-northeast-2:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'ap-northeast-3': {
    arm64: 'arn:aws:lambda:ap-northeast-3:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:ap-northeast-3:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'ap-south-1': {
    arm64: 'arn:aws:lambda:ap-south-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:ap-south-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'ap-southeast-1': {
    arm64: 'arn:aws:lambda:ap-southeast-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:ap-southeast-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'ap-southeast-2': {
    arm64: 'arn:aws:lambda:ap-southeast-2:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:ap-southeast-2:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'ca-central-1': {
    arm64: 'arn:aws:lambda:ca-central-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:ca-central-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'eu-central-1': {
    arm64: 'arn:aws:lambda:eu-central-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:eu-central-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'eu-north-1': {
    arm64: 'arn:aws:lambda:eu-north-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:eu-north-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'eu-west-1': {
    arm64: 'arn:aws:lambda:eu-west-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:eu-west-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'eu-west-2': {
    arm64: 'arn:aws:lambda:eu-west-2:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:eu-west-2:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'eu-west-3': {
    arm64: 'arn:aws:lambda:eu-west-3:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:eu-west-3:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'sa-east-1': {
    arm64: 'arn:aws:lambda:sa-east-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:sa-east-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'us-east-1': {
    arm64: 'arn:aws:lambda:us-east-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:us-east-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'us-east-2': {
    arm64: 'arn:aws:lambda:us-east-2:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:us-east-2:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'us-west-1': {
    arm64: 'arn:aws:lambda:us-west-1:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:us-west-1:354918379484:layer:optimeist-extension-x86_64:1',
  },
  'us-west-2': {
    arm64: 'arn:aws:lambda:us-west-2:354918379484:layer:optimeist-extension-arm64:1',
    x86_64: 'arn:aws:lambda:us-west-2:354918379484:layer:optimeist-extension-x86_64:1',
  },
}
