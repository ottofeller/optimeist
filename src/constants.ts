import * as cdk from 'aws-cdk-lib'
import layersData from './layers.json'

interface LayerInfo {
    arm64: string
    x86_64: string
}

export const layersMap: Record<typeof cdk.Aws.REGION, LayerInfo> = layersData
