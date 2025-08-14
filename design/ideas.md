## Rolling weight transfer

// Multiple inference servers
GPU 0: Still serving with version N
GPU 1: Downloading version N+1  
GPU 2: Quantizing version N+1
// When GPU 1&2 ready, they take traffic, GPU 0 updates