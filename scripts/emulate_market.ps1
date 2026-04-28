param(
    [int]$Steps = 300,
    [int]$Seed = 42,
    [int]$DelayMs = 40,
    [string]$BaseUrl = "http://127.0.0.1:8080"
)

$ErrorActionPreference = "Stop"

Get-Random -SetSeed $Seed | Out-Null

for ($attempt = 1; $attempt -le 30; $attempt++) {
    try {
        Invoke-RestMethod -Method Get -Uri "$BaseUrl/health" | Out-Null
        break
    }
    catch {
        if ($attempt -eq 30) {
            throw "API is not reachable at $BaseUrl after waiting"
        }
        Start-Sleep -Milliseconds 500
    }
}

$marketBody = @{
    base = "BTC"
    quote = "USDT"
} | ConvertTo-Json

Invoke-RestMethod -Method Post -Uri "$BaseUrl/markets" -ContentType "application/json" -Body $marketBody | Out-Null

function Submit-Order {
    param(
        [string]$Side,
        [string]$OrderType,
        [Nullable[long]]$Price,
        [long]$Qty,
        [long]$UserId
    )

    $payload = @{
        market = @{
            base = "BTC"
            quote = "USDT"
        }
        user_id = $UserId
        side = $Side
        order_type = $OrderType
        price = $Price
        qty = $Qty
    } | ConvertTo-Json -Depth 5

    Invoke-RestMethod -Method Post -Uri "$BaseUrl/orders" -ContentType "application/json" -Body $payload
}

# initial liquidity
$user = 1
$center = 100000
for ($i = 1; $i -le 12; $i++) {
    $qty = ($i % 5) + 2
    $bidPrice = $center - ($i * 20)
    $askPrice = $center + ($i * 20)
    Submit-Order -Side "bid" -OrderType "limit" -Price $bidPrice -Qty $qty -UserId $user | Out-Null
    $user++
    Submit-Order -Side "ask" -OrderType "limit" -Price $askPrice -Qty $qty -UserId $user | Out-Null
    $user++
}

Write-Output "API SIM START steps=$Steps seed=$Seed market=BTC/USDT"
$referencePrice = 100000

for ($step = 1; $step -le $Steps; $step++) {
    $drift = Get-Random -Minimum -50 -Maximum 51
    $referencePrice = [Math]::Max(90000, $referencePrice + $drift)

    $actionRoll = Get-Random -Minimum 0 -Maximum 100
    $side = if ((Get-Random -Minimum 0 -Maximum 2) -eq 0) { "bid" } else { "ask" }
    $orderType = if ($actionRoll -lt 30) { "market" } else { "limit" }
    $qty = if ($orderType -eq "market") {
        Get-Random -Minimum 1 -Maximum 6
    } else {
        Get-Random -Minimum 1 -Maximum 9
    }

    $price = $null
    if ($orderType -eq "limit") {
        $edge = Get-Random -Minimum 5 -Maximum 121
        if ($side -eq "bid") {
            $price = $referencePrice - $edge
        } else {
            $price = $referencePrice + $edge
        }
    }

    $report = Submit-Order -Side $side -OrderType $orderType -Price $price -Qty $qty -UserId $user
    $user++

    $snap = Invoke-RestMethod -Method Get -Uri "$BaseUrl/markets/BTC/USDT/snapshot?depth=5"
    $bestBid = $snap.best_bid
    $bestAsk = $snap.best_ask
    $spread = $null
    if ($null -ne $bestBid -and $null -ne $bestAsk) {
        $spread = $bestAsk - $bestBid
    }

    $line = "step={0:d4} side={1} type={2} filled={3} rem={4} trades={5} last={6} bid={7} ask={8} spread={9}" -f `
        $step, $side, $orderType, $report.filled_qty, $report.remaining_qty, $report.trades.Count, `
        $report.last_price_after, $bestBid, $bestAsk, $spread
    Write-Output $line

    Start-Sleep -Milliseconds $DelayMs
}

Write-Output "API SIM DONE"
