import { Block } from "casper-js-sdk"

export function getCurrentBlockHeight(block: Block): number {
    if ("Version2" in block) {
        return block.Version2.header.height
    } else if ("Version1" in block) {
        return block.Version1.header.height
    }
    throw new Error("Unknown block version")
}
