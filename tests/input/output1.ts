
/**
 */
export type AType = number;

/**
 * Doc-comments for a type def
 */
export type BType = number;

/**
 * Doc-comment line 1 for A
 * Doc-comment line 2 for A
 * Doc-comment line 3 for A
 */
export type A = {
    /**
     */
    a1_field: U64;

    /**
     */
    a2_field: U64;

    /**
     * Line for a3
     * Line for a2, then blank line
     * 
     * Some markdown
     * ```
     * const a = [];
     * const b = "";
     * ```
     */
    a3_field: U128;

}

/**
 */
export type B = {
    /**
     */
    b: U64;

}

/**
 * doc-comment for enum
 */
export enum E {
    /**
     */
    V1,

    /**
     */
    V2,

}

/**
 */
export interface C {
    /**
     * init func
     */
    init_here: { f128: U128 };

    /**
     * Line 1 for get_f128 first
     * Line 2 for get_f128 second
     */
    get_f128(): Promise<U128>;

    /**
     * Set f128.
     */
    set_f128(args: { value: U128 }, gas?: any): Promise<void>;

    /**
     */
    get_f128_other_way(args: { key: U128 }): Promise<U128>;

    /**
     */
    more_types(args: { key: U128, tuple: [string, number[]] }, gas?: any): Promise<void>;

    /**
     * Pay to set f128.
     */
    set_f128_with_sum(args: { a_value: U128, other_value: U128 }, gas?: any, amount?: any): Promise<void>;

}

/**
 */
export interface C {
    /**
     * another impl
     */
    another_impl(args: { f128: U128 }): Promise<U128>;

}

/**
 */
export interface I {
    /**
     * Single-line comment for get
     */
    get(): Promise<U128>;

}

/**
 */
export type A_in_mod = number;

export interface C extends I {}
