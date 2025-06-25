import * as cdk from 'aws-cdk-lib'
import {layersMap} from './constants'
import {Construct} from 'constructs'

export const getExtensionLayerArn = (
  scope: Construct,
  params: {
    architecture: cdk.aws_lambda.Architecture
  },
): string => {
  const cfnArchKey = params.architecture === cdk.aws_lambda.Architecture.ARM_64 ? 'arm64' : 'x8664' // Remove underscore

  const layerArnMapping = new cdk.CfnMapping(scope, 'OptimeistLayerArnMapping', {
    mapping: Object.entries(layersMap).reduce(
      (acc, [regionKey, archValues]) => {
        acc[regionKey] = {
          arm64: archValues.arm64,
          x8664: archValues.x86_64,
        }
        return acc
      },
      {} as Record<string, Record<string, string>>,
    ),
  })

  return layerArnMapping.findInMap(cdk.Aws.REGION, cfnArchKey)
}
