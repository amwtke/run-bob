package com.example.shared.usecase;

/**
 * 通用 UseCase 接口。所有 usecase 类必须 implements 此接口。
 *
 * 不允许 import 任何框架代码(Spring / Jakarta / SLF4J / Lombok)。
 *
 * @param <C> Command 类型(usecase 包内的 record)
 * @param <R> Result 类型(usecase 包内的 record)
 */
public interface UseCase<C, R> {
    R execute(C cmd);
}
