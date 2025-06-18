import * as cdk from 'aws-cdk-lib'
import {LAYER_ACCOUNT_ID} from './constants'

export const getExtensionLayerArn = (params: {
  version: number
  layerName: string
  architecture: cdk.aws_lambda.Architecture
}): string => {
  const architecture =
    params.architecture?.dockerPlatform === cdk.aws_lambda.Architecture.ARM_64.dockerPlatform ? 'arm64' : 'x86_64'

  const partition = cdk.Aws.PARTITION
  const region = 'us-east-1' // FIXME Use cdk.Aws.REGION when lambda layers are available in all regions

  return `arn:${partition}:lambda:${region}:${LAYER_ACCOUNT_ID}:layer:${params.layerName}-${architecture}:${params.version}`
}
