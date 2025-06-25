import * as cdk from 'aws-cdk-lib'
import {layersMap} from './constants'

export const getExtensionLayerArn = (params: {architecture: cdk.aws_lambda.Architecture}): string => {
  const architecture =
    params.architecture?.dockerPlatform === cdk.aws_lambda.Architecture.ARM_64.dockerPlatform ? 'arm64' : 'x86_64'

  return layersMap[cdk.Aws.REGION][architecture]
}
