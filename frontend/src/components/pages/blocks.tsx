import { useState, useEffect } from "react"
import { Flex, VStack, Text, Button, Spinner, Box } from "@chakra-ui/react"
import { Helmet } from "react-helmet-async"
import BlockRowElement from "../molecules/blocks-row-element"
import { Block, GetBlockResult } from "casper-js-sdk"
import { useIsNetworkRunningContext } from "../../context/IsNetworkRunningContext"
import { defaultClient } from "../../casper-client"
import { getCurrentBlockHeight } from "../atoms/block-utils"

const Blocks: React.FC = () => {
    const [blocks, setBlocks] = useState<any[]>([]) // TODO: This really needs to be of type BlockV1[] | BlockV2[]
    const [loading, setLoading] = useState<boolean>(true)
    const [error, setError] = useState<string | null>(null)
    const [currentPage, setCurrentPage] = useState<number>(1)
    const [isLastPage, setIsLastPage] = useState<boolean>(false)
    const { isNetworkRunning } = useIsNetworkRunningContext()

    const DISPLAY_PER_PAGE = 10

    useEffect(() => {
        const fetchBlocks = async () => {
            setLoading(true)
            setError(null)
            try {
                let latestBlockInfo
                try {
                    latestBlockInfo = await defaultClient.casperService.getLatestBlockInfo()
                    console.log(latestBlockInfo) //
                } catch (error) {
                    console.error("Error fetching latest block info:", error)
                    setBlocks([])
                    setIsLastPage(true)
                    return
                }

                if (!latestBlockInfo || !latestBlockInfo.block_with_signatures?.block) {
                    setBlocks([])
                    setIsLastPage(true)
                    return
                }

                const block = latestBlockInfo.block_with_signatures.block

                const currentHeight = getCurrentBlockHeight(block)

                const blockHeights = []
                for (let i = 0; i < DISPLAY_PER_PAGE; i++) {
                    const height = currentHeight - i - (currentPage - 1) * DISPLAY_PER_PAGE
                    if (height >= 0) {
                        blockHeights.push(height)
                    }
                }

                if (blockHeights.length === 0) {
                    setBlocks([])
                    setIsLastPage(true)
                    return
                } else {
                    setIsLastPage(false)
                }

                const blockInfoPromises = blockHeights.map((height) =>
                    defaultClient.casperService.getBlockInfoByHeight(height)
                )
                const blockInfos: GetBlockResult[] = await Promise.all(blockInfoPromises)
                console.log(blockInfos) //
                const newBlocks = blockInfos
                    .map((blockInfo) => blockInfo.block_with_signatures?.block)
                    .filter((block): block is Block => block !== null)
                    .map((block) => {
                        if ("Version2" in block) {
                            return block.Version2
                        } else if ("Version1" in block) {
                            return block.Version1
                        }
                        throw new Error("Unexpected block type")
                    })

                console.log(newBlocks) //
                setBlocks(newBlocks)
                setIsLastPage(newBlocks.length < DISPLAY_PER_PAGE)
            } catch (err) {
                console.error("Error in fetchBlocks:", err)
                setError(err instanceof Error ? err.message : String(err))
            } finally {
                setLoading(false)
            }
        }
        if (isNetworkRunning) {
            fetchBlocks()
        }
    }, [currentPage, isNetworkRunning])

    const handlePrevious = () => {
        setCurrentPage((prevPage) => Math.max(prevPage - 1, 1))
    }

    const handleNext = () => {
        if (!isLastPage) {
            setCurrentPage((prevPage) => prevPage + 1)
        }
    }

    if (loading) {
        return (
            <Flex
                justifyContent="center"
                height="calc(100vh - 148px)"
                alignItems="center"
                mt={["144px", "144px", "0"]}
            >
                {isNetworkRunning ? (
                    <Spinner size="xl" colorScheme="gray" />
                ) : (
                    <Text color="grey.400">Network paused</Text>
                )}
            </Flex>
        )
    }

    if (error) {
        return (
            <Flex w="100%" justify="center" color="grey.400" pt="100px">
                <Text>Error fetching blocks: {error}</Text>
            </Flex>
        )
    }

    if (blocks.length === 0 && !loading) {
        return (
            <Flex justifyContent="center" height="calc(100vh - 148px)" alignItems="center">
                <Box overflowY="auto" p={3}>
                    <Flex w="100%" justify="center" mt={["144px", "144px", "0"]}>
                        <Text color="grey.400">No blocks available to display</Text>
                    </Flex>
                </Box>
            </Flex>
        )
    }

    return (
        <>
            <Helmet>
                <title>Fondant | Blocks</title>
            </Helmet>
            <Flex w="100%" justify="center" m={["140px 0 0 0", "140px 0 0 0", "80px 0 0 0"]}>
                <VStack w="100%" maxW="1440px" gap="0">
                    {blocks.map((block) => (
                        <BlockRowElement
                            key={block.hash}
                            height={block.header.height}
                            era={block.header.era_id}
                            transactions={Object.keys(block.body.transactions).length}
                            age={block.header.timestamp}
                            blockHash={block.hash}
                        />
                    ))}
                </VStack>
            </Flex>
            <Flex justify="center" mt="4" mb="5" gap="15px">
                <Button onClick={handlePrevious} isDisabled={currentPage === 1} w="90px">
                    Previous
                </Button>
                <Button onClick={handleNext} isDisabled={isLastPage} w="90px">
                    Next
                </Button>
            </Flex>
        </>
    )
}

export default Blocks
