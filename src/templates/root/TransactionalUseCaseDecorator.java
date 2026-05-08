package com.example.shared.framework.transaction;

import com.example.shared.usecase.UseCase;
import org.springframework.transaction.annotation.Transactional;

/**
 * 全工程唯一的 @Transactional 所在地。
 *
 * 用法:在 framework/config/<Feature>UseCaseConfig.java:
 *
 *   @Bean
 *   UseCase<MyCommand, MyResult> myUseCase(MyRepository repo, ...) {
 *       return new TransactionalUseCaseDecorator<>(
 *           new MyUseCase(repo, ...));
 *   }
 *
 * 命令、查询统一走装饰器,无例外。
 */
public class TransactionalUseCaseDecorator<C, R> implements UseCase<C, R> {

    private final UseCase<C, R> inner;

    public TransactionalUseCaseDecorator(UseCase<C, R> inner) {
        this.inner = inner;
    }

    @Override
    @Transactional
    public R execute(C cmd) {
        return inner.execute(cmd);
    }
}
