| :rocket: `init_here` (*constructor*) |  init func | `Self` |
| :eyeglasses: `get_f128` |  Line 1 for get_f128 first  Line 2 for get_f128 second | `U128` |
| :writing_hand: `set_f128` |  Set f128. | `void` |
| :eyeglasses: `get_f128_other_way` |  | `U128` |
| :writing_hand: `more_types` |  | `void` |
| &#x24C3; `set_f128_with_sum` |  Pay to set f128. | `void` |
| :eyeglasses: `another_impl` |  another impl | `U128` |
| :eyeglasses: `get` |  Single-line comment for get | `U128` |


## Methods for C

### :rocket: `init_here` (*constructor*)

```typescript
init_here: { f128: U128 };
```

init func

### :eyeglasses: `get_f128`

```typescript
get_f128(): Promise<U128>;
```

Line 1 for get_f128 first
Line 2 for get_f128 second

### :writing_hand: `set_f128`

```typescript
set_f128(args: { value: U128 }, gas?: any): Promise<void>;
```

Set f128.

### :eyeglasses: `get_f128_other_way`

```typescript
get_f128_other_way(args: { key: U128 }): Promise<U128>;
```


### :writing_hand: `more_types`

```typescript
more_types(args: { key: U128, tuple: [string, number[]] }, gas?: any): Promise<void>;
```


### &#x24C3; `set_f128_with_sum`

```typescript
set_f128_with_sum(args: { a_value: U128, other_value: U128 }, gas?: any, amount?: any): Promise<void>;
```

Pay to set f128.

## Methods for C

### :eyeglasses: `another_impl`

```typescript
another_impl(args: { f128: U128 }): Promise<U128>;
```

another impl

## Methods for `I` interface

### :eyeglasses: `get`

```typescript
get(): Promise<U128>;
```

Single-line comment for get
