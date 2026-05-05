# AtCoder Submissions Backend

## API

### `GET /submissions`
URL Parameters
- `contest: string`
- `task?: string`
- `user?: string`
- `language?: string`
- `status?: "AC" | "WA" | "TLE" | "MLE" | "RE" | "CE" | "QLE" | "OLE" | "IE"`
- `order_by?: "timestamp" | "score" | "code_size" | "execution_time" | "memory_usage"`
- `order_desc?: boolean`
- `offset?: number`
- `count?: number`

Result: JSON
```js
{
    error?: string,
    result?: [
        {
            task: string,
            user: string,
            language: string,
            timestamp: number,
            status: string,
            code_size: number,
            score?: number,
            execution_time?: number,
            memory_usage?: number,
        }
    ],
}
```